//! <https://rust-windowing.github.io/winit/winit/index.html>

pub use winit::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error creating window {0:?}")]
    EventLoopError(#[from] winit::error::EventLoopError),
}

#[derive(Debug, smart_default::SmartDefault)]
enum State {
    #[default]
    Uninit,
    Init,
    Running,
    Suspended,
    Exited,
}

#[derive(Debug)]
pub(crate) struct WinitApplication {
    /// There is a chance there is a need for more than one window. It is highly unlikely, but we
    /// have to account for it
    windows: smallvec::SmallVec<[Window; 1]>,
    input: super::EventLoopInput,
    state: State,
}
impl super::ApplicationHandler for WinitApplication {}

impl winit::application::ApplicationHandler for WinitApplication {
    fn new_events(&mut self, event_loop: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        let _ = (event_loop, cause);
        match cause {
            event::StartCause::WaitCancelled {
                start: _,
                requested_resume: _,
            }
            | event::StartCause::ResumeTimeReached {
                start: _,
                requested_resume: _,
            } => {
                self.state = State::Running;
            }
            event::StartCause::Poll | event::StartCause::Init => {
                self.state = State::Init;
            }
        }
    }

    fn user_event(&mut self, event_loop: &event_loop::ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        device_id: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
        self.state = State::Suspended;
    }

    fn exiting(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
        self.state = State::Exited;
    }

    fn memory_warning(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        match self.state {
            State::Uninit => todo!(),
            State::Init => {
                self.windows = self
                    .input
                    .app
                    .get_window_descriptions()
                    .into_iter()
                    .map(|x| event_loop.create_window(x.into()).unwrap().into())
                    .collect();
                self.state = State::Running;
            }
            State::Running => todo!(),
            State::Suspended => todo!(),
            State::Exited => todo!(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: event::WindowEvent,
    ) {
        use winit::event::WindowEvent;
        let window_id: super::WindowID = window_id.into();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.windows[0].as_ref().request_redraw();
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Context {
    event_loop: event_loop::EventLoop<()>,
}

impl super::PlatformlessContext for Context {
    type WindowType = Window;
    type ApplicationType = WinitApplication;

    fn run(self, inp: super::EventLoopInput) -> Result<super::Output, super::Error> {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        self.event_loop
            .set_control_flow(winit::event_loop::ControlFlow::Poll);
        // // ControlFlow::Wait pauses the event loop if no events are available to process.
        // // This is ideal for non-game applications that only update in response to user
        // // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        // self.event_loop.set_control_flow(ControlFlow::Wait);
        let mut app = WinitApplication {
            windows: smallvec::SmallVec::new(),
            input: inp,
            state: State::Uninit,
        };
        if let Err(e) = self.event_loop.run_app(&mut app) {
            return Err(Error::EventLoopError(e).into());
        }
        Ok(super::Output {})
    }

    fn build() -> Self {
        use winit::event_loop::EventLoop;
        let event_loop = EventLoop::new().unwrap();
        Self { event_loop }
    }
}

#[derive(Debug, derive_more::Deref)]
pub(crate) struct Window {
    window: Box<winit::window::Window>,
}

impl From<winit::window::Window> for Window {
    fn from(window: winit::window::Window) -> Self {
        Self {
            window: Box::new(window),
        }
    }
}

impl From<winit::window::WindowId> for super::WindowID {
    fn from(val: winit::window::WindowId) -> Self {
        Self(val.into())
    }
}

impl super::WindowHandler for Window {
    fn get_raw_handle(&self) {
        todo!()
    }
    fn get_id(&self) -> super::WindowID {
        self.window.id().into()
    }
}

impl From<super::WindowAttributes> for winit::window::WindowAttributes {
    fn from(val: super::WindowAttributes) -> Self {
        let mut r = Self::default()
            .with_title(val.title)
            .with_transparent(val.transparent)
            .with_decorations(val.decorations)
            .with_resizable(val.resizable)
            .with_maximized(val.maximized)
            .with_visible(val.visible);
        if let Some(size) = val.inner_size {
            r = r.with_inner_size(size);
        }
        if let Some(size) = val.min_inner_size {
            r = r.with_min_inner_size(size);
        }
        if let Some(size) = val.max_inner_size {
            r = r.with_max_inner_size(size);
        }
        r
    }
}

impl From<super::Size> for winit::dpi::Size {
    fn from(val: super::Size) -> Self {
        Self::Physical(winit::dpi::PhysicalSize::new(val.x, val.y))
    }
}
/// Size of a window stored in pixels
pub type Size = nalgebra::Vector2<u32>;
/// Offset on monitor in pixels
pub type Position = nalgebra::Vector2<u32>;
