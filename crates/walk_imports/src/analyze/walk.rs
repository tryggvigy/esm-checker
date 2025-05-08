use super::{types::AnalysisError, Analysis};
use crate::analyze::{has_cjs_syntax::has_cjs_syntax, parse::parse};
use es_resolver::{errors::ResolveError, prelude::*, utils::get_npm_package_name};
use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use swc_core::{
    common::{sync::Lrc, SourceMap},
    ecma::loader::NODE_BUILTINS,
};
use swc_ecma_dep_graph::{analyze_dependencies, DependencyKind};
use tracing::{debug, error, trace, warn};

pub fn walk(
    current_module: &str,
    import_path: &Path,
    // import_specifier: &str,
    entrypoint: &Path,
    node_resolver: &impl Resolve,
    code_map: &Lrc<SourceMap>,
    analysis: &mut Analysis,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), AnalysisError> {
    trace!("Walking imports for {:?}", entrypoint);

    if visited.contains(entrypoint) {
        // TODO investigate why it happens so often? something wrong?
        trace!(
            "Already visited: \"{:?}\" in \"{:?}\"",
            entrypoint,
            import_path
        );
        return Ok(());
    }
    visited.insert(entrypoint.to_owned());
    //
    // Skip .json files or .node files
    match entrypoint.extension().and_then(OsStr::to_str) {
        Some("json") | Some("node") => return Ok(()),
        _ => {}
    }

    let (module, comments) =
        parse(code_map, entrypoint).map_err(|e| AnalysisError::ParseError {
            package_name: analysis.package_name.clone(),
            path: entrypoint.to_owned(),
            original_error_message: e.to_string(),
        })?;

    let has_cjs = has_cjs_syntax(&module);
    if has_cjs {
        debug!("Found CommonJS syntax in {:?}", entrypoint);
        // TODO what if transitive dep of react imports react as well?
        if current_module == analysis.package_name {
            analysis.is_entry_esm = false;
        } else {
            analysis
                .transitive_commonjs_dependencies
                .insert(current_module.to_string());
        }
    }

    let dependencies = analyze_dependencies(&module, &comments);
    let filtered_deps = dependencies
        .iter()
        .filter(|dependency| {
            matches!(
                dependency.kind,
                DependencyKind::Import | DependencyKind::Export | DependencyKind::Require
            )
        })
        .collect::<Vec<_>>();

    for dep in filtered_deps {
        let mut specifier = dep.specifier.as_ref();
        let original_specifier = specifier;
        let mut allow_node_builtins = true;

        if let Some(base) = specifier.strip_suffix('/') {
            // This is used e.g. for `string_decoder/`. The trailing slash is
            // used to opt out of the Node.js builtin.
            specifier = base;
            allow_node_builtins = false;
        }

        if specifier.starts_with('.') && !specifier.ends_with(".js") && !specifier.ends_with(".mjs")
        {
            analysis
                .esm_missing_js_file_extensions
                .insert(current_module.to_string());
        }

        // Skip processing node built-ins and json files.
        if specifier.starts_with("node:") || specifier.ends_with(".json") {
            continue;
        }

        // If the specifier is not a relative path, we are entering a new module.
        // set it to the specifier.
        let new_current_module = if !specifier.starts_with('.') {
            get_npm_package_name(specifier)
        } else {
            current_module
        };

        let resolved_dependency = match node_resolver.resolve(specifier.to_string(), entrypoint) {
            Ok(resolved_path_buf) => resolved_path_buf,
            Err(_) if allow_node_builtins && NODE_BUILTINS.contains(&specifier) => {
                continue;
            }
            Err(ResolveError::PeerDependencyNotInstalled(peer_dependency_name)) => {
                warn!(
                    "Skipping not installed peer dependency: {}",
                    peer_dependency_name
                );
                continue;
            }
            Err(e) => {
                error!(
                    "Failed to resolve {:?} from {:?}: {:?}",
                    original_specifier.to_string(),
                    entrypoint,
                    e
                );
                return Err(AnalysisError::ResolveError {
                    package_name: analysis.package_name.clone(),
                    import_specifier: original_specifier.to_string(),
                    from: entrypoint.to_path_buf(),
                    source: Box::new(e),
                });
            }
        };

        walk(
            new_current_module,
            entrypoint,
            &resolved_dependency,
            node_resolver,
            code_map,
            analysis,
            visited,
        )?;
    }

    Ok(())
}
