//! https://rust-windowing.github.io/winit/winit/index.html

use std::ops::Deref;
use winit::*;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Error creating window {0:?}")]
    EventLoopError(#[from] winit::error::EventLoopError),
}

#[derive(Debug)]
struct WinitApplication {
    /// There is a chance there is a need for more than one window. It is highly unlikely, but we
    /// have to account for it
    windows: smallvec::SmallVec<[Window; 1]>,
    app: super::EventLoopInput,
}
impl super::ApplicationHandler for WinitApplication {}

impl winit::application::ApplicationHandler for WinitApplication {
    fn new_events(&mut self, event_loop: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        let _ = (event_loop, cause);
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
    }

    fn exiting(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: event::WindowEvent,
    ) {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Context {
    event_loop: event_loop::EventLoop<()>,
}

impl super::PlatformlessContext for Context {
    type WindowType = Window;
    type ApplicationType = WinitApplication;

    fn run(&self, inp: super::EventLoopInput) -> Result<super::Output, super::Error> {
        use winit::event_loop::*;
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        self.event_loop.set_control_flow(ControlFlow::Poll);
        // // ControlFlow::Wait pauses the event loop if no events are available to process.
        // // This is ideal for non-game applications that only update in response to user
        // // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        // self.event_loop.set_control_flow(ControlFlow::Wait);
        let mut app = WinitApplication {
            windows: smallvec::SmallVec::new(),
            app: inp,
        };
        self.event_loop.run_app(&mut app)?.into();
        Ok(super::Output {})
    }

    fn build() -> Self {
        use winit::event_loop::*;
        let event_loop = EventLoop::new().unwrap();
        Self { event_loop }
    }
}

#[derive(Debug)]
struct Window {
    window: Box<winit::window::Window>,
}

impl Into<super::WindowID> for winit::window::WindowId {
    fn into(self) -> super::WindowID {
        super::WindowID(self.into())
    }
}

impl super::WindowHandler for Window {
    fn get_raw_handle(&self) -> () {
        todo!()
    }
    fn get_id(&self) -> super::WindowID {
        self.window.id().into()
    }
}

// impl ApplicationHandler for App {
//     fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
//         self.window = Some(event_loop.create_window(WindowAttributes::default()).unwrap());
//     }
//
//     fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, id: WindowId, event: WindowEvent) {
//         match event {
//             WindowEvent::CloseRequested => {
//                 println!("The close button was pressed; stopping");
//                 event_loop.exit();
//             },
//             WindowEvent::RedrawRequested => {
//                 // Redraw the application.
//                 //
//                 // It's preferable for applications that do not render continuously to render in
//                 // this event rather than in AboutToWait, since rendering in here allows
//                 // the program to gracefully handle redraws requested by the OS.
//
//                 // Draw.
//
//                 // Queue a RedrawRequested event.
//                 //
//                 // You only need to call this if you've determined that you need to redraw in
//                 // applications which do not always need to. Applications that redraw continuously
//                 // can render here instead.
//                 self.window.as_ref().unwrap().request_redraw();
//             }
//             _ => (),
//         }
//     }
// }
