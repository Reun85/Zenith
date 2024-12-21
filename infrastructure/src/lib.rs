#![feature(negative_impls)]
// Start of code
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(clippy::perf, clippy::suspicious, clippy::style)]
#![allow(dead_code)]
pub mod resource_management;
pub mod undroppable;
pub mod unsafe_ref;
pub use resource_management::*;
pub mod promise;
pub use promise::*;

pub trait Flags {
    /// Should be used for single flag
    fn has_flags(&self, flags: &Self) -> bool;
    /// Should be used for single flag
    #[must_use]
    fn set_flags(&self, flags: &Self) -> Self;
    /// Should be used for single flag
    #[must_use]
    fn rem_flags(&self, flags: &Self) -> Self;
}

pub trait StateConstains {
    fn contains(&self, other: &Self) -> bool;
}
