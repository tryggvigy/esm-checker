use es_resolver::errors::ResolveError;
use std::{collections::BTreeSet, path::PathBuf};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Analysis {
    pub package_name: String,
    pub is_entry_esm: bool,
    pub transitive_commonjs_dependencies: BTreeSet<String>,
    pub esm_missing_js_file_extensions: BTreeSet<String>,
}

#[derive(Debug, Error)]
pub enum AnalysisError {
    /// Failed to resolve a module.
    #[error("Failed to resolve module {0} from {1}", .import_specifier, .from.display())]
    ResolveError {
        package_name: String,
        import_specifier: String,
        from: PathBuf,
        #[source]
        source: Box<ResolveError>,
    },
    /// A file failed to parse.
    #[error("Failed to parse file {0}: {1}", .path.display(), .original_error_message)]
    ParseError {
        package_name: String,
        path: PathBuf,
        original_error_message: String,
    },
}
