use std::{path::Path, sync::Arc};

use crate::{
    package_json::{PackageJson, PackageJsonParser},
    resolve_chain::{ChainStep, ResolveStepResult},
    utils::get_npm_package_name,
};

/// "Resolve" step that parses the `package.json` file and returns it as state
/// for the next step(s).
pub struct PackageJsonResolver {
    parser: Arc<PackageJsonParser>,
}

impl PackageJsonResolver {
    /// Create a new `PackageJsonResolver`, using the given `package.json` parser.
    pub fn new(parser: Arc<PackageJsonParser>) -> Self {
        Self { parser }
    }
}

impl<Input> ChainStep<Input, Arc<PackageJson>> for PackageJsonResolver {
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        _state: Input,
    ) -> ResolveStepResult<Arc<PackageJson>> {
        // Crawl up until we find a `node_modules` folder.
        let mut module_path = match self.parser.find_node_modules(from) {
            Ok(p) => p,
            Err(e) => return ResolveStepResult::Error(e),
        };

        let package_name = get_npm_package_name(&import_specifier);
        module_path.push(package_name);

        match self
            .parser
            .get_or_parse_package_json(module_path, Some(package_name.to_owned()))
        {
            Ok(package_json) => ResolveStepResult::Continue(import_specifier, package_json),
            Err(err) => ResolveStepResult::Error(err),
        }
    }
}
