use std::{path::Path, sync::Arc};

use crate::{
    errors::ResolveError,
    package_json::PackageJsonParser,
    resolve_chain::{ChainStep, ResolveStepResult},
    utils::ImplicitFileResolver,
};

/// Resolver that handles relative file imports.
pub struct RelativePathResolver<'a> {
    package_json_parser: Arc<PackageJsonParser>,
    implicit_file_resolver: Option<ImplicitFileResolver<'a>>,
}

impl<'a> RelativePathResolver<'a> {
    /// Create a new [`RelativePathResolver`], using the given `package.json` parser.
    pub fn new(
        package_json_parser: Arc<PackageJsonParser>,
        implicit_file_resolver: Option<ImplicitFileResolver<'a>>,
    ) -> Self {
        Self {
            package_json_parser,
            implicit_file_resolver,
        }
    }
}

impl<'a, Input> ChainStep<Input, Input> for RelativePathResolver<'a> {
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        state: Input,
    ) -> ResolveStepResult<Input> {
        if !import_specifier.starts_with('.') {
            return ResolveStepResult::Continue(import_specifier, state);
        }

        let Some(containing_directory) = from.parent() else {
            return ResolveStepResult::Error(ResolveError::FromPathHasNoParent);
        };

        let path = containing_directory.join(&import_specifier);
        if path.is_file() {
            return ResolveStepResult::Ok(path);
        }
        if let Some(implicit_file_resolver) = &self.implicit_file_resolver {
            if let Some(path) = implicit_file_resolver.try_resolve_implicitly(path.clone()) {
                return ResolveStepResult::Ok(path);
            }
        }

        // This might be a relative path crawling up to the root of a package.
        // If so, replace the import specifier with the package name.
        let possible_package_json = path.join(PackageJsonParser::PACKAGE_JSON);
        if possible_package_json.is_file() {
            // Parse the package.json and replace the import specifier with the main field.
            let package_json = match self
                .package_json_parser
                .get_or_parse_package_json(path.clone(), None)
            {
                Ok(p) => p,
                Err(e) => return ResolveStepResult::Error(e),
            };

            if let Some(package_name) = package_json.name.as_ref() {
                return ResolveStepResult::Continue(package_name.clone(), state);
            }
        }

        ResolveStepResult::Error(ResolveError::FileNotFound(path))
    }
}
