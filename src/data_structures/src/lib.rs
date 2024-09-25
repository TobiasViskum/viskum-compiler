use std::{ fmt::Debug, ops::{ Deref, DerefMut } };

use fxhash::FxBuildHasher;
use indexmap::IndexSet;

/// A wrapper of IndexSet using FxHash instead
pub struct FxIndexSet<T>(IndexSet<T, FxBuildHasher>);

impl<T> Default for FxIndexSet<T> {
    fn default() -> Self {
        Self(IndexSet::default())
    }
}

impl<T> Deref for FxIndexSet<T> {
    type Target = IndexSet<T, FxBuildHasher>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FxIndexSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
