use std::{path::Path, sync::Arc};

use crate::{
    package_json::PackageJson,
    resolve_chain::{ChainStep, ResolveStepResult},
    utils::ImplicitFileResolver,
};

/// Resolver that checks for the existence of a file in a package, e.g. check
/// for `bar/baz.js` in package `foo` when the import specifier is `foo/bar/baz`,
/// or `foo/bar/baz.js`.
pub struct FileResolver<'a> {
    implicit_file_resolver: Option<ImplicitFileResolver<'a>>,
}

impl<'a> FileResolver<'a> {
    /// Create a new [`FileResolver`].
    pub fn new(implicit_file_resolver: Option<ImplicitFileResolver<'a>>) -> Self {
        Self {
            implicit_file_resolver,
        }
    }
}

impl<'a> ChainStep<Arc<PackageJson>, Arc<PackageJson>> for FileResolver<'a> {
    fn call(
        &self,
        import_specifier: String,
        _from: &Path,
        state: Arc<PackageJson>,
    ) -> ResolveStepResult<Arc<PackageJson>> {
        if let Some(package_name) = state.name.as_ref() {
            if let Some(sub_path) = import_specifier.strip_prefix(&format!("{}/", package_name)) {
                let path = state.package_root.join(sub_path);
                if path.is_file() {
                    return ResolveStepResult::Ok(path);
                }
                if let Some(implicit_file_resolver) = &self.implicit_file_resolver {
                    if let Some(path) = implicit_file_resolver.try_resolve_implicitly(path) {
                        return ResolveStepResult::Ok(path);
                    }
                }
            }
        }

        ResolveStepResult::Continue(import_specifier, state)
    }
}
