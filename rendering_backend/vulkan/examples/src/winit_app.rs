//! <https://rust-windowing.github.io/winit/winit/index.html>

use std::rc::Rc;
use window::WindowAttributes;
pub use winit::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    EventLoopError(#[from] winit::error::EventLoopError),
    #[error(transparent)]
    OsError(#[from] winit::error::OsError),
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

pub struct InitInfo<'a> {
    pub window: &'a winit::window::Window,
}
pub struct ShutdownInfo {}
pub struct RenderInfo<'a> {
    pub window: &'a winit::window::Window,
}
pub struct EventInfo {
    pub event: winit::event::WindowEvent,
}

pub(crate) trait Application {
    fn init(info: InitInfo) -> Self;
    fn render(&mut self, info: RenderInfo);
    fn window_event(&mut self, info: EventInfo);
    /// Precedes dropping the application
    fn shutdown(&mut self, info: ShutdownInfo);
}

#[derive(Debug)]
pub(crate) struct WinitApplication<App: Application> {
    app: Option<App>,
    state: State,
    window: Option<Rc<winit::window::Window>>,
}

impl<T: Application> WinitApplication<T> {
    pub fn run() -> anyhow::Result<()> {
        use winit::event_loop::EventLoop;
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = WinitApplication::<T> {
            app: None,
            state: State::Uninit,
            window: None,
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

impl<T: Application> winit::application::ApplicationHandler for WinitApplication<T> {
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
        if let Some(mut x) = self.app.take() {
            let info = ShutdownInfo {};
            x.shutdown(info);
        }
    }

    fn memory_warning(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        match self.state {
            State::Uninit => todo!(),
            State::Init => {
                self.state = State::Running;

                match event_loop.create_window(WindowAttributes::default()) {
                    Ok(x) => {
                        self.window = Some(Rc::new(x));
                        let info = InitInfo {
                            window: self.window.as_ref().unwrap(),
                        };
                        self.app = Some(T::init(info));
                    }
                    Err(e) => {
                        eprintln!("Error creating window: {:?}", e);
                        event_loop.exit();
                    }
                }
            }
            State::Running => todo!(),
            State::Suspended => todo!(),
            State::Exited => todo!(),
        }
    }

    fn window_event(
        &mut self,
        _: &event_loop::ActiveEventLoop,
        // Only 1 window
        _: window::WindowId,
        event: event::WindowEvent,
    ) {
        use winit::event::WindowEvent;
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(x) = self.app.as_mut() {
                    let info = RenderInfo {
                        window: self.window.as_ref().unwrap(),
                    };

                    x.render(info);
                }
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
                self.window.as_ref().unwrap().request_redraw();
            }

            _ => {
                if let Some(x) = self.app.as_mut() {
                    let info = EventInfo { event };
                    x.window_event(info);
                }
            }
        }
    }
}
