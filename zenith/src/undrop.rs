
struct Helper<const DROPPABLE: bool>;
impl<const DROPPABLE: bool> Drop for Helper<DROPPABLE> {
    fn drop(&mut self) {
        struct ConstBlock<const D: bool>;
        impl<const D: bool> ConstBlock<D> {
            const BLOCK: () = {
                if !D {
                    panic!("type containing `PhantomUnDrop` cannot be simply dropped.");
                }
            };
        }
        let _ = ConstBlock::<DROPPABLE>::BLOCK;
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
pub const PhantomUndrop: PhantomUnDrop = PhantomUnDrop { helper: Helper };
