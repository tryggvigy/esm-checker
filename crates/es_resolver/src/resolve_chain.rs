//! This module contains all the machinery for wiring up the resolvers into a
//! chain that can be used to resolve ES module imports.

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::errors::ResolveError;

/// The intermediate result of a step in the resolve chain.
#[derive(Debug)]
pub enum ResolveStepResult<T> {
    /// The import specifier was resolved to a path.
    Ok(PathBuf),
    /// Continue looking for the given import specifier, and with the given
    /// state.
    Continue(String, T),
    /// An error occurred during resolution. Resolving should stop.
    Error(ResolveError),
}

impl<T> From<ResolveError> for ResolveStepResult<T> {
    fn from(error: ResolveError) -> Self {
        ResolveStepResult::Error(error)
    }
}

/// A step in the chain, either implemented on a struct in the resolve chain, or
/// a function pointer that satisfies the correct signature.
pub trait ChainStep<Input, Output> {
    /// Call the step in the chain. The step should first call any previous steps
    /// in the chain, and then do its own work. If the previous step returns
    /// [`ResolveStepResult::Ok`], the step should return that result. If the
    /// previous step returns [`ResolveStepResult::Continue`], the step should
    /// do its own work, using the import specifier and state from the previous
    /// step. If the previous step returns [`ResolveStepResult::Error`], the
    /// step should return that error.
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        state: Input,
    ) -> ResolveStepResult<Output>;
}

/// Type alias for a resolve function.
pub type ResolveFunction<Input, Output> = fn(String, &Path, Input) -> ResolveStepResult<Output>;

// Allow using function pointers as steps in the resolve chain.
impl<Input, Output> ChainStep<Input, Output> for ResolveFunction<Input, Output> {
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        state: Input,
    ) -> ResolveStepResult<Output> {
        (self)(import_specifier, from, state)
    }
}

/// One link in the chain of resolvers.
pub struct Chain<InitialInput, Input, Prev, F> {
    prev: Prev,
    f: F,
    _t: PhantomData<InitialInput>,
    _u: PhantomData<Input>,
}

/// Utility function to bootstrap a new chain.
/// This is essentially just a dummy resolver that never resolves anything, but
/// that you can chain onto.
fn new_chain_fn(import_specifier: String, _from: &Path, state: ()) -> ResolveStepResult<()> {
    ResolveStepResult::Continue(import_specifier, state)
}

/// Utility function to bootstrap a new chain.
/// This is essentially just a dummy resolver that never resolves anything, but
/// that you can chain onto.
#[allow(non_upper_case_globals)]
pub const new_chain: ResolveFunction<(), ()> = new_chain_fn;

/// A trait for the chain of resolvers. Allows calling into the chain to resolve
/// an import specifier, and chaining onto this item to add another resolver.
pub trait ResolveChain<InitialInput, Input> {
    /// Call into the chain to resolve an import specifier. The chain will
    /// either return a path, or a new import specifier and state to continue
    /// resolving.
    /// If the chain returns an error, resolving should stop.
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use es_resolver::prelude::*;
    /// use es_resolver::resolve_chain::ResolveStepResult;
    ///
    /// fn resolve<T>(resolver: &impl ResolveChain<(), T>, import_specifier: String, from: &Path) -> ResolveStepResult<T> {
    ///   let result = resolver.call(import_specifier, from, ());
    ///   match &result {
    ///     ResolveStepResult::Ok(path) => {
    ///       println!("Resolved to path: {:?}", path);
    ///     },
    ///     ResolveStepResult::Continue(import_specifier, state) => {
    ///       println!("Continue resolving with import specifier: {:?}", import_specifier);
    ///     },
    ///     ResolveStepResult::Error(error) => {
    ///       println!("Error: {:?}", error);
    ///     },
    ///   }
    ///   result
    /// }
    /// ```
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        input: InitialInput,
    ) -> ResolveStepResult<Input>;

    /// Add an item to the chain, to be called after this item. The `next` item
    /// added will be responsible for calling this item during execution.
    fn chain<Output, F>(self, next: F) -> Chain<InitialInput, Input, Self, F>
    where
        Self: Sized,
        F: ChainStep<Input, Output>,
    {
        Chain {
            prev: self,
            f: next,
            _t: PhantomData,
            _u: PhantomData,
        }
    }
}

/// Implement the [`ResolveChain`] trait for function pointers, to allow chaining
/// onto this item.
impl<InitialInput, Input> ResolveChain<InitialInput, Input>
    for fn(String, &Path, InitialInput) -> ResolveStepResult<Input>
{
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        input: InitialInput,
    ) -> ResolveStepResult<Input> {
        self(import_specifier, from, input)
    }
}

impl<InitialInput, Input, Output, Prev, F> ResolveChain<InitialInput, Output>
    for Chain<InitialInput, Input, Prev, F>
where
    Prev: ResolveChain<InitialInput, Input>,
    F: ChainStep<Input, Output>,
{
    fn call(
        &self,
        import_specifier: String,
        from: &Path,
        input: InitialInput,
    ) -> ResolveStepResult<Output> {
        match self.prev.call(import_specifier, from, input) {
            ResolveStepResult::Ok(p) => ResolveStepResult::Ok(p),
            ResolveStepResult::Continue(import_specifier, state) => {
                self.f.call(import_specifier, from, state)
            }
            ResolveStepResult::Error(e) => ResolveStepResult::Error(e),
        }
    }
}
