/// Default constraints for types
// Will include the Zenith specific viewer later
pub trait VortexDebug
where
    Self: std::fmt::Debug,
{
}

impl<T> VortexDebug for T where T: std::fmt::Debug {}

pub trait Flags
where
    Self: VortexDebug,
{
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
    fn delete(&mut self, resource: &Resource);
}

/// Used for a owning resourceReference where deleting the resource requires larger context, that
/// owner provides
#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct ResourceRef<Value, Owner: ResourceDeleter<Value>> {
    #[deref]
    #[deref_mut]
    pub value: Value,
    pub owner: std::rc::Rc<std::cell::RefCell<Owner>>,
}

impl<Value, Owner: ResourceDeleter<Value>> Drop for ResourceRef<Value, Owner> {
    fn drop(&mut self) {
        self.owner.borrow_mut().delete(&self.value);
    }
}

struct Helper<const DROPPABLE: bool>;
impl<const DROPPABLE: bool> Drop for Helper<DROPPABLE> {
    fn drop(&mut self) {
        struct ConstBlock<const D: bool>;
        impl<const D: bool> ConstBlock<D> {
            const BLOCK: () = {
                assert!(
                    D,
                    "type containing `PhantomUnDrop` cannot be simply dropped."
                );
            };
        }
        // This is here so that the panic happens at compile time.
        #[allow(clippy::let_unit_value)]
        let _: () = ConstBlock::<DROPPABLE>::BLOCK;
    }
}
// A marker type that cannot be dropped. `free` must be called on it.
pub struct PhantomUnDrop {
    helper: Helper<false>,
}

impl PhantomUnDrop {
    pub fn free(self) {
        let Self { helper } = self;
        let droppable: Helper<true> = unsafe { std::mem::transmute(helper) };
        std::mem::drop(droppable);
    }
}
#[allow(non_upper_case_globals)]
pub const PhantomUnDrop: PhantomUnDrop = PhantomUnDrop { helper: Helper };

pub mod unsafe_ref;
