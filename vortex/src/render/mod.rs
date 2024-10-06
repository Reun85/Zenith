//! This module contains all the rendering logic for the engine.

#[cfg(feature = "use-vulkan")]
pub mod vulkan_platform;

#[cfg(feature = "use-vulkan")]
pub use crate::render::vulkan_platform as platform_impl;

/// The usable context for the rendering engine
// It may be just a way to access the C API or a may hold physical data
pub struct Context(platform_impl::Context);

trait ContextLike {
    fn new() -> Self;
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    PlatformSpecific(#[from] platform_impl::GenericError),
}
