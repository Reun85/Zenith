use winit::*;

struct Context {
    event_loop: event_loop::EventLoop<()>,
    /// There is a chance there is a need for more than one window. It is highly unlikely, but we
    /// have to account for it
    windows: smallvec::SmallVec<[WindowHandler; 1]>,
}

impl super::Context for Context {
    type Window = WindowHandler;
    type Output = ();
    fn new() -> Self {
        let event_loop = event_loop::EventLoop::builder().build().unwrap();
        Self {
            event_loop,
            windows: smallvec::SmallVec::new(),
        }
    }

    fn create_window(&mut self) -> Self::Window {
        WindowHandler {}
    }
    fn run(&mut self, inp: super::EventLoopInput) -> Self::Output {}
}

struct WindowHandler {}
impl super::Window for WindowHandler {}

fn temp() {}
