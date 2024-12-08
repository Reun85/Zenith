#![feature(negative_impls)]
// Start of code
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(clippy::perf, clippy::suspicious, clippy::style)]
#![allow(dead_code)]
pub mod undroppable;
pub mod unsafe_ref;
/// Default constraints for types
// Will include the Zenith specific viewer later
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

pub trait ResourceDeleter<Resource> {
    fn delete(&mut self, resource: &mut Resource);
}
impl<Resource, Owner> ResourceDeleter<Resource> for Owner
where
    Owner: AsMut<dyn ResourceDeleter<Resource>>,
{
    fn delete(&mut self, resource: &mut Resource) {
        self.as_mut().delete(resource);
    }
}

/// Used for a owning resourceReference where deleting the resource requires larger context, that
/// owner provides
#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct ResourceRef<Value, Owner: ResourceDeleter<Value>> {
    #[deref]
    #[deref_mut]
    pub value: Value,
    pub owner: Owner,
}

impl<Value, Owner: ResourceDeleter<Value>> Drop for ResourceRef<Value, Owner> {
    fn drop(&mut self) {
        self.owner.delete(&mut self.value);
    }
}

#[derive(Debug, derive_more::Deref, derive_more::DerefMut, derive_more::Into, Clone, Copy)]
pub struct Promise<Result, Description = (), Intermediary = ()> {
    #[deref]
    #[into]
    #[deref_mut]
    pub result: Option<Result>,
    pub description: Description,
    pub intermediary: Intermediary,
}

impl<R, D, I> Promise<R, D, I>
where
    I: std::default::Default,
{
    pub fn new(desc: D) -> Self {
        Self {
            result: None,
            description: desc,
            intermediary: I::default(),
        }
    }
    pub fn unwrap(self) -> R {
        self.result.unwrap()
    }
    pub fn result(self) -> Option<R> {
        self.result
    }
}
