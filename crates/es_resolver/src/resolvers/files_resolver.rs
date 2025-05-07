use std::{path::Path, sync::Arc};

use crate::{package_json::PackageJson, resolve_chain::ResolveStepResult};

/// Resolver that checks the `files` field in `package.json` to see if there
/// is an `index.js`-like file listed.
pub fn files_resolver(
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
        if let Some(files) = &state.raw.files {
            for possible_file in ["index.js", "index.cjs"] {
                if files.contains(&possible_file.to_owned()) {
                    return ResolveStepResult::Ok(state.package_root.join(possible_file));
                }
            }
        }
    }

    ResolveStepResult::Continue(import_specifier, state)
}
