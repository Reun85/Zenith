//! # Vortex

// Start of code
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

extern crate tracing;
extern crate tracing_subscriber;
extern crate winit;

pub mod build_constants;
pub mod event;
pub mod infrastructure;
pub mod log;
pub mod undrop;
pub mod window;
use window::EventLoopLike;

// If the result is an Err its fine to use box as this will definitely lead to a shutdown.
pub trait UserApplication
where
    Self: crate::infrastructure::Debug,
{
    /// Run per game update
    /// Currently unused
    fn update(&mut self) {}

    /// Run per each frame update
    fn render(&mut self) {}

    fn on_init(&mut self, context: &mut window::EventLoop) {}

    ///
    fn on_window_event(
        &mut self,
        event: Box<dyn event::EventLike<Category = window::input::EventCategories>>,
    ) {
        let _ = event;
    }

    /// This function is always preceded by [WindowCloseEvents](`event::WindowCloseEvent`) to
    /// [on_window_event](`Self::on_window_event`)
    fn on_exit(&mut self) {}
}

pub trait UserApplicationBuilder {
    type Application: UserApplication;
    fn new() -> Self::Application;
}

impl<T: Default + UserApplication> UserApplicationBuilder for T {
    type Application = T;

    fn new() -> Self::Application {
        Self::default()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    LoggerError(#[from] log::Error),
    #[error(transparent)]
    WindowManager(#[from] window::Error),
}

fn start<App: UserApplicationBuilder>() -> Result<window::Output, Error>
where
    <App as UserApplicationBuilder>::Application: 'static,
{
    log::init_logging()?;
    let app = {
        let _s = log::debug_span!("Init application");

        Box::new(App::new())
    };

    {
        let window_context = window::EventLoop::build();
        let ev_inp = window::EventLoopInput { app };
        let output = window_context.run(ev_inp)?;
        Ok(output)
    }
}
