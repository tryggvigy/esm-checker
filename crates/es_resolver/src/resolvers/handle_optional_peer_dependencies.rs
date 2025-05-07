use std::sync::Arc;

use crate::{
    errors::ResolveError,
    package_json::PackageJsonParser,
    resolve_chain::{ChainStep, ResolveStepResult},
    utils::get_npm_package_name,
};

/// Resolver that checks if the import specifier is an optional peer dependency.
/// If it is, and the package is not installed, it will return a specific error code,
/// that can be used to skip the package.
pub struct HandleOptionalPeerDependenciesResolver {
    package_json_parser: Arc<PackageJsonParser>,
}

impl HandleOptionalPeerDependenciesResolver {
    /// Create a new instance of the resolver, using the given [`PackageJsonParser`].
    pub fn new(package_json_parser: Arc<PackageJsonParser>) -> Self {
        Self {
            package_json_parser,
        }
    }
}

impl<Input> ChainStep<Input, Input> for HandleOptionalPeerDependenciesResolver {
    fn call(
        &self,
        import_specifier: String,
        from: &std::path::Path,
        state: Input,
    ) -> crate::resolve_chain::ResolveStepResult<Input> {
        let from_directory = if from.is_dir() {
            from
        } else if let Some(parent) = from.parent() {
            parent
        } else {
            return ResolveStepResult::Error(ResolveError::FromPathHasNoParent);
        };

        // Find the package.json file for `from`
        let package_json = match self.package_json_parser.find_package_json(from_directory) {
            Ok(path) => path,
            Err(err) => return err.into(),
        };
        let package_json = match self.package_json_parser.get_or_parse_package_json(
            package_json
                .parent()
                .expect("package_json has no parent directory")
                .to_path_buf(),
            None,
        ) {
            Ok(package_json) => package_json,
            Err(err) => return err.into(),
        };

        // Check if the import specifier is an optional peer dependency.
        let import_specifier_package_name = get_npm_package_name(&import_specifier);
        if package_json
            .raw
            .peer_dependencies
            .as_ref()
            .map(|deps| deps.contains_key(import_specifier_package_name))
            .unwrap_or(false)
        {
            if let Some(meta) = package_json
                .raw
                .peer_dependencies_meta
                .as_ref()
                .and_then(|meta| meta.get(import_specifier_package_name))
            {
                if meta.optional {
                    // Check if the package is installed. Otherwise, return a specific error code.
                    let mut module_path = match self.package_json_parser.find_node_modules(from) {
                        Ok(p) => p,
                        Err(e) => return e.into(),
                    };
                    module_path.push(import_specifier_package_name);

                    if !module_path.exists() {
                        return ResolveError::PeerDependencyNotInstalled(
                            import_specifier_package_name.to_string(),
                        )
                        .into();
                    }
                }
            }
        }

        ResolveStepResult::Continue(import_specifier, state)
    }
}
