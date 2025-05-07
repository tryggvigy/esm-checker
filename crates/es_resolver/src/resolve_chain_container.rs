//! A container for a resolver chain. This is the entrypoint into the resolver
//! chain, and is the only type that needs to be exposed to the user, when using
//! the [`crate::presets`](pre-made resolver chains).

use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{
    errors::ResolveError,
    resolve_chain::{Chain, ChainStep, ResolveChain, ResolveStepResult},
};

/// A container that holds a resolver chain.
pub struct Resolver<Input, Output, Prev, F> {
    chain: Chain<(), Input, Prev, F>,
    output: PhantomData<Output>,
}

impl<Input, Output, Prev, F> Resolver<Input, Output, Prev, F> {
    /// Create a new [`Resolver`], using the given resolver chain.
    pub fn new(chain: Chain<(), Input, Prev, F>) -> Self {
        Self {
            chain,
            output: PhantomData,
        }
    }
}

/// An opaque entrypoint into the resolver chain. This allows hiding the internal types of the
/// resolver chain, which (due to generics) get pretty gnarly.
pub trait Resolve {
    /// Resolve an import specifier into a path.
    fn resolve(&self, import_specifier: String, from: &Path) -> Result<PathBuf, ResolveError>;
}

impl<Input, Output, Prev, F> Resolve for Resolver<Input, Output, Prev, F>
where
    Prev: ResolveChain<(), Input>,
    F: ChainStep<Input, Output>,
{
    fn resolve(&self, import_specifier: String, from: &Path) -> Result<PathBuf, ResolveError> {
        match self.chain.call(import_specifier, from, ()) {
            ResolveStepResult::Ok(p) => Ok(fs::canonicalize(p).map_err(|e| {
                ResolveError::CanonicalizeRelativePathFailed(from.to_path_buf(), e)
            })?),
            ResolveStepResult::Continue(import_specifier, _) => Err(ResolveError::FailedToResolve(
                import_specifier,
                from.to_owned(),
            )),
            ResolveStepResult::Error(e) => Err(e),
        }
    }
}
