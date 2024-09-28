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
use window::PlatformlessContext;

pub mod build_constants;
pub mod event;
pub mod infrastructure;
pub mod log;
pub mod undrop;
pub mod window;

// If the result is an Err its fine to use box as this will definitely lead to a shutdown.
pub trait UserApplication
where
    Self: crate::infrastructure::Debug,
{
    /// Post engine initialization
    fn init(&mut self) {}

    /// Run per game update
    fn update(&mut self) {}
    /// Run per each frame update
    fn render(&mut self) {}

    /// Runs when the event loop is exiting
    fn exit(&mut self) {}

    fn get_title(&self) -> String;
    fn get_window_descriptions(&self) -> Vec<window::WindowAttributes> {
        vec![window::WindowAttributes {
            title: self.get_title(),
            ..window::WindowAttributes::default()
        }]
    }
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

/// # Errors
/// Will return a non recoverable error
pub fn start_application<App: UserApplicationBuilder>() -> Result<window::Output, Error>
where
    <App as UserApplicationBuilder>::Application: 'static,
{
    log::init_logging()?;
    let app = {
        let _s = log::debug_span!("Init application");

        Box::new(App::new())
    };

    {
        let window_manager = window::Context::build();
        let ev_inp = window::EventLoopInput { app };
        let output = window_manager.run(ev_inp)?;
        Ok(output)
    }
}
