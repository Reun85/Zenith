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

pub mod build_constants;
pub mod event;
pub mod infrastructure;
pub mod log;
pub mod undrop;
pub mod window;

type UserError = Box<dyn std::error::Error>;
// If the result is an Err its fine to use box as this will definitely lead to a shutdown.
pub trait UserApplication
where
    Self: Sized,
{
    /// Pre engine initialization
    fn new() -> Result<Self, UserError>;

    /// Post engine initialization
    fn init(&mut self) {}

    /// Run per game update
    fn update(&mut self) {}
    /// Run per each frame update
    fn render(&mut self) {}

    /// Runs when the event loop is exiting
    fn exit(&mut self) {}
}

#[derive(Debug, thiserror::Error)]
pub enum Error<App: UserApplication> {
    #[error(transparent)]
    External(#[from] UserApplication::Error),
    #[error(transparent)]
    LoggerError(#[from] log::Error),
}

pub fn start_application(app: Box<dyn UserApplication>) -> Result<(), Error<App>> {
    log::init_logging()?;
    let mut app = {
        let _s = log::debug_span!("Init application");

        match App::new() {
            Ok(x) => Box::new(x),
            Err(e) => return Err(Error::External(UserError::BuildError(e))),
        }
    };

    {
        let window_manager = window::Context::build();
        let ev_inp = window::EventLoopInput { app };
        let output = window_manager.run(ev_inp)?;
    }

    Ok(())
}
