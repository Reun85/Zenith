#[cfg(feature = "winit")]
pub mod winit;
#[cfg(feature = "winit")]
pub use winit as platform_impl;

mod input;

// TODO: Give this a better name
#[derive(Debug)]
pub(crate) struct EventLoopInput {
    pub(crate) app: Box<dyn super::UserApplication>,
}

#[derive(Debug)]
pub(crate) struct Output {}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    PlatformSpecific(#[from] platform_impl::Error),
}

pub trait PlatformlessContext {
    type ApplicationType: ApplicationHandler;
    type WindowType: WindowHandler;

    fn build() -> Self;
    /// The main EventLoop of the application resides here
    fn run(&self, inp: EventLoopInput) -> Result<Output, Error>;
}

#[derive(Debug)]
pub(crate) struct Context(platform_impl::Context);

impl PlatformlessContext for Context {
    type WindowType = <platform_impl::Context as PlatformlessContext>::WindowType;
    type ApplicationType = <platform_impl::Context as PlatformlessContext>::ApplicationType;
    fn build() -> Self {
        Self(platform_impl::Context::build())
    }
    fn run(&self, inp: EventLoopInput) -> Result<Output, Error> {
        self.0.run(inp)
    }
}

/// Specifically does not require `std::hash::Hash` as most games will have 1 window, using a
/// HashMap is overkill
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct WindowID(u64);

pub(crate) trait WindowHandler {
    fn get_raw_handle(&self) -> ();
    fn get_id(&self) -> WindowID;
}

pub(crate) trait ApplicationHandler {}
