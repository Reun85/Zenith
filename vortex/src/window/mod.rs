#[cfg(feature = "winit")]
pub mod winit;

mod input;

pub(crate) struct EventLoopInput {}

pub(crate) trait Context {
    type Window: Window;
    type Output;
    fn new() -> Self;

    fn create_window(&mut self) -> Self::Window;
    fn run(&mut self, inp: EventLoopInput) -> Self::Output;
}

pub(crate) trait Window {}
