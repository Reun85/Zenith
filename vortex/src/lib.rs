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

pub trait UserApplication
where
    Self: Sized,
{
    type BuildError: crate::infrastructure::Debug + Sized;

    /// Pre engine initialization
    fn new() -> Result<Self, Self::BuildError>;

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
pub enum UserError<App: UserApplication> {
    #[error("User Application build returned with error {0:?}")]
    BuildError(App::BuildError),
}
#[derive(Debug, thiserror::Error)]
pub enum Error<App: UserApplication> {
    #[error(transparent)]
    External(#[from] UserError<App>),
    #[error(transparent)]
    LoggerError(#[from] log::Error),
}

pub fn start_application<App: UserApplication>() -> Result<(), Error<App>> {
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
