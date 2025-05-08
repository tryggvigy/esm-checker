use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use es_resolver::package_json::PackageJsonParser;
use es_resolver::prelude::*;
use swc_core::common::{sync::Lrc, SourceMap};
use tracing::info;

use crate::analyze::walk::walk;

use super::types::{Analysis, AnalysisError};

pub fn analyze_package(
    path: &Path,
    package_name: &str,
    package_json_parser: &PackageJsonParser,
    node_resolver: &impl Resolve,
) -> Result<Analysis, AnalysisError> {
    info!("Processing {}", package_name);

    let mut module_path = path.join("node_modules");
    module_path.push(package_name);

    let package_json = package_json_parser
        .get_or_parse_package_json(module_path, Some(package_name.to_owned()))
        .map_err(|e| AnalysisError::ResolveError {
            package_name: package_name.to_string(),
            import_specifier: package_name.to_string(),
            from: path.to_path_buf(),
            source: Box::new(e),
        })?;
    let code_map: Lrc<SourceMap> = Default::default();

    let mut analysis = Analysis {
        package_name: package_name.to_string(),
        is_entry_esm: true,
        transitive_commonjs_dependencies: BTreeSet::new(),
        esm_missing_js_file_extensions: BTreeSet::new(),
    };

    let mut visited = HashSet::new();

    for entrypoint in package_json
        .get_entrypoints(&presets::get_default_condition_names(), node_resolver)
        .map_err(|e| AnalysisError::ResolveError {
            package_name: package_name.to_string(),
            import_specifier: package_name.to_string(),
            from: path.to_path_buf(),
            source: Box::new(e),
        })?
    {
        walk(
            package_name,
            path,
            &entrypoint,
            node_resolver,
            &code_map,
            &mut analysis,
            &mut visited,
        )?;
    }

    Ok(analysis)
}
