#![deny(missing_docs)]

//! This crate contains resolvers that can be used to resolve ES module imports. The [`presets`]
//! module contains some pre-made chains that use these resolvers, but you can create your own
//! chains if you want. You can even incorporate your own resolvers if you want, by implementing
//! the [`crate::resolve_chain::ChainStep`] trait.
//!
//! # Example
//! ```
//! use es_resolver::prelude::*;
//! use std::path::Path;
//!
//! let resolver = presets::get_default_es_resolver();
//! let resolved = resolver.resolve("foo".to_string(), &Path::new("/path/to/file.js"));
//! ```

pub mod errors;
pub mod package_json;
pub mod presets;
pub mod resolve_chain;
pub mod resolve_chain_container;
pub mod resolvers;
#[cfg(test)]
mod tests;
pub mod utils;

/// A prelude that re-exports the most commonly used types from this crate.
pub mod prelude {
    pub use crate::presets;
    pub use crate::resolve_chain::{ResolveChain, ResolveFunction};
    pub use crate::resolve_chain_container::Resolve;
}
