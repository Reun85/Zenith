#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

#[macro_export]
macro_rules! bitflags_to_vortex_flags {
    ($t:tt) => {
        impl vortex::infrastructure::Flags for $t {
            fn has_flags(&self, flags: &$t) -> bool {
                self.contains(*flags)
            }
            fn set_flags(&mut self, flags: &$t) {
                self.insert(*flags);
            }
            fn intersects(&self, flags: &$t) -> bool {
                self.intersects(*flags)
            }
        }
    };
}
