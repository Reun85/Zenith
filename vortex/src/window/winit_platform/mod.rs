//! <https://rust-windowing.github.io/winit/winit/index.html>

pub use winit::*;

use crate::window::input;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error creating window {0:?}")]
    EventLoopError(#[from] winit::error::EventLoopError),
    #[error("{0}")]
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

#[derive(Debug, PartialEq, Eq, derive_more::From)]
pub struct DeviceID(winit::event::DeviceId);

#[derive(Debug)]
pub(crate) struct WinitApplication {
    /// There is a chance there is a need for more than one window. It is highly unlikely, but we
    /// have to account for it
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
                self.state = State::Running;
                self.input.app = Some(
                    (self.input.app_creater)(&mut super::InitContext(InitContext { event_loop }))
                        .unwrap(),
                );
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
                let ev = input::Event::Window(input::window::Event::WindowClose(
                    input::window::CloseEvent { id: window_id },
                ));
                if let Some(x) = self.input.app.as_mut() {
                    x.on_window_event(&ev);
                }
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let ev = input::Event::Window(input::window::Event::WindowResize(
                    input::window::ResizeEvent {
                        id: window_id,
                        size: size.into(),
                    },
                ));
                if let Some(x) = self.input.app.as_mut() {
                    x.on_window_event(&ev);
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let device_id = DeviceID(device_id);

                let ev = match state {
                    event::ElementState::Pressed => {
                        input::Event::Mouse(input::mouse::Event::MousePress(input::mouse::Press {
                            device_id: device_id.into(),
                            button: button.into(),
                            // TODO: this is unused
                            repeat: false,
                        }))
                    }
                    event::ElementState::Released => input::Event::Mouse(
                        input::mouse::Event::MouseRelease(input::mouse::Release {
                            device_id: device_id.into(),
                            button: button.into(),
                        }),
                    ),
                };
                if let Some(x) = self.input.app.as_mut() {
                    x.on_window_event(&ev);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(x) = self.input.app.as_mut() {
                    x.render();
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
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
pub(crate) struct EventLoop {
    event_loop: event_loop::EventLoop<()>,
}

#[derive(Debug)]
pub struct InitContext<'a> {
    event_loop: &'a winit::event_loop::ActiveEventLoop,
}

impl<'a> super::InitContextLike for InitContext<'a> {
    fn create_window(
        &mut self,
        attributes: super::WindowAttributes,
    ) -> Result<super::Window, super::Error> {
        match self.event_loop.create_window(attributes.into()) {
            Ok(x) => Ok(Into::<Window>::into(x).into()),
            Err(e) => Err(Into::<Error>::into(e).into()),
        }
    }
}

impl super::EventLoopLike for EventLoop {
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

#[derive(Debug)]
pub struct Window(winit::window::Window);

impl raw_window_handle::HasDisplayHandle for Window {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.0.display_handle()
    }
}

impl raw_window_handle::HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.0.window_handle()
    }
}

impl From<winit::window::Window> for Window {
    fn from(value: winit::window::Window) -> Self {
        Self(value)
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
        self.0.id().into()
    }

    fn inner_size(&self) -> super::PhysicalSize {
        self.0.inner_size().into()
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

impl From<winit::event::MouseButton> for crate::window::input::mouse::Button {
    fn from(val: winit::event::MouseButton) -> Self {
        match val {
            winit::event::MouseButton::Left => crate::window::input::mouse::Button::Left,
            winit::event::MouseButton::Right => crate::window::input::mouse::Button::Right,
            winit::event::MouseButton::Middle => crate::window::input::mouse::Button::Middle,
            winit::event::MouseButton::Back => crate::window::input::mouse::Button::Back,
            winit::event::MouseButton::Forward => crate::window::input::mouse::Button::Forward,
            winit::event::MouseButton::Other(x) => crate::window::input::mouse::Button::Other(x),
        }
    }
}

impl From<super::PhysicalSize> for winit::dpi::Size {
    fn from(val: super::PhysicalSize) -> Self {
        Self::Physical(winit::dpi::PhysicalSize::new(val.width, val.height))
    }
}
impl From<winit::dpi::PhysicalSize<u32>> for super::PhysicalSize {
    fn from(val: winit::dpi::PhysicalSize<u32>) -> Self {
        nalgebra::Vector2::<u32>::new(val.width, val.height).into()
    }
}
