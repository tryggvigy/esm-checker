//! Errors that can occur while resolving import specifiers.

use std::{io, path::PathBuf};
use thiserror::Error;

/// An error that occurred while resolving an import specifier.
#[derive(Debug, Error)]
pub enum ResolveError {
    /// Failed to canonicalize a relative path.
    #[error("Failed to canonicalize relative path {0}: {1}")]
    CanonicalizeRelativePathFailed(PathBuf, io::Error),
    /// Failed to resolve an import specifier: reached the end of the resolve
    /// chain without successfully resolving the specifier.
    #[error("Failed to resolve {0} from {1}")]
    FailedToResolve(String, PathBuf),
    /// The import specifier refers to a file that does not exist.
    #[error("File {0} not found")]
    FileNotFound(PathBuf),
    /// The path that we're resolving from has no parent.
    #[error("From path has no parent")]
    FromPathHasNoParent,
    /// Encountered an IO error while resolving an import specifier.
    #[error("Encountered IO error at {0}: {1}")]
    IoError(PathBuf, io::Error),
    /// The `node_modules` directory could not be found.
    #[error("Unable to locate node_modules directory")]
    NodeModulesNotFound,
    /// The `package.json` file for the current package could not be found.
    #[error("Unable to locate package.json for {0}")]
    PackageJsonNotFound(PathBuf),
    /// Failed to parse a `package.json` file.
    #[error("Failed to parse package.json {0}: {1:?}")]
    ParsePackageJsonFailed(PathBuf, serde_json::Error),
    /// The import specifier referred to a peer dependency that was not installed.
    #[error("The import specifier referred to peer dependency {0} that was not installed")]
    PeerDependencyNotInstalled(String),
}
