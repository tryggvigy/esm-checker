use std::{path::Path, sync::Arc};

use crate::{
    package_json::{PackageJson, PackageJsonParser},
    resolve_chain::{ChainStep, ResolveStepResult},
};

/// Handles imports to packages such as `dom-helpers` users. The imports are in the form of
/// `dom-helpers/addClass`, where `addClass` is a folder in the `dom-helpers` package containing a
/// `package.json` file with fields such as `main` and `module`.
///
/// Replaces the resolved `package.json` file with the one for the pseudo package.
pub struct PseudoNamespaceResolver {
    package_json_parser: Arc<PackageJsonParser>,
}

impl PseudoNamespaceResolver {
    /// Create a new `PseudoNamespaceResolver`, using the given `package.json` parser.
    pub fn new(package_json_parser: Arc<PackageJsonParser>) -> Self {
        Self {
            package_json_parser,
        }
    }
}

impl ChainStep<Arc<PackageJson>, Arc<PackageJson>> for PseudoNamespaceResolver {
    fn call(
        &self,
        import_specifier: String,
        _from: &Path,
        state: Arc<PackageJson>,
    ) -> ResolveStepResult<Arc<PackageJson>> {
        if let Some((scope, subpath)) = import_specifier.split_once('/') {
            if !scope.starts_with('@') && !subpath.contains('/') {
                // This is the type of package we're looking for in this resolver.
                // First, we need to check if the package exists.
                let module_path = state.package_root.join(subpath);

                let package_json_path = module_path.join(PackageJsonParser::PACKAGE_JSON);
                if package_json_path.is_file() {
                    // Try to get the package.json file for this "package".
                    if let Ok(package_json) = self.package_json_parser.get_or_parse_package_json(
                        module_path,
                        Some(format!("{}/{}", scope, subpath)),
                    ) {
                        return ResolveStepResult::Continue(import_specifier, package_json);
                    }
                }
            }
        }

        ResolveStepResult::Continue(import_specifier, state)
    }
}
