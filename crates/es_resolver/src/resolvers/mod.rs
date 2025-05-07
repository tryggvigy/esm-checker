//! This module contains resolvers that can be used in a
//! [`ResolveChain`](`crate::resolve_chain::ResolveChain`). The [`presets`](`crate::presets`)
//! module contains some pre-made chains that use these resolvers, but you can create your own
//! chains if you want. You can even incorporate your own resolvers if you want, by implementing
//! the [`ChainStep`](`crate::resolve_chain::ChainStep`) trait.
//!
//! # Example
//!
//! ```
//! use es_resolver::package_json::{PackageJson, PackageJsonParser};
//! use es_resolver::prelude::*;
//! use es_resolver::resolve_chain::{new_chain, ResolveStepResult};
//! use es_resolver::resolvers::*;
//!
//! use std::path::Path;
//! use std::sync::Arc;
//!
//! fn my_custom_resolver(
//!   import_specifier: String,
//!   from: &Path,
//!   state: Arc<PackageJson>
//! ) -> ResolveStepResult<Arc<PackageJson>> {
//!   todo!("Implement your own resolver here")
//! }
//!
//! let package_json_parser = Arc::new(PackageJsonParser::new());
//! let resolve_chain = new_chain
//!     .chain(RelativePathResolver::new(Arc::clone(&package_json_parser), None))
//!     .chain(PackageJsonResolver::new(package_json_parser))
//!     .chain(index_resolver as ResolveFunction<_, _>)
//!     .chain(FileResolver::new(None))
//!     .chain(my_custom_resolver as ResolveFunction<_, _>);
//! ```

mod exports_resolver;
mod file_resolver;
mod files_resolver;
mod handle_optional_peer_dependencies;
mod index_resolver;
mod package_json_resolver;
mod pseudo_namespace_resolver;
mod relative_path_resolver;

pub use exports_resolver::{ExportsResolver, FieldName};
pub use file_resolver::FileResolver;
pub use files_resolver::files_resolver;
pub use handle_optional_peer_dependencies::HandleOptionalPeerDependenciesResolver;
pub use index_resolver::index_resolver;
pub use package_json_resolver::PackageJsonResolver;
pub use pseudo_namespace_resolver::PseudoNamespaceResolver;
pub use relative_path_resolver::RelativePathResolver;
