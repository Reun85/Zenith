// TODO: Add cfg for platform specific windowing

#[cfg(feature = "winit")]
pub mod winit;
#[cfg(feature = "winit")]
pub use winit::*;

trait WindowHandler {}
