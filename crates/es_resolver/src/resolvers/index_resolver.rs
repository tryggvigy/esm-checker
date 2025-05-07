use std::{path::Path, sync::Arc};

use crate::{package_json::PackageJson, resolve_chain::ResolveStepResult};

/// Resolver that just checks if there's an index.js file in the root of the
/// package.
pub fn index_resolver(
    import_specifier: String,
    _from: &Path,
    state: Arc<PackageJson>,
) -> ResolveStepResult<Arc<PackageJson>> {
    if state
        .name
        .as_ref()
        .map(|name| name == &import_specifier)
        .unwrap_or(false)
    {
        let path = state.package_root.join("index.js");
        if path.is_file() {
            return ResolveStepResult::Ok(path);
        }
    }

    ResolveStepResult::Continue(import_specifier, state)
}
