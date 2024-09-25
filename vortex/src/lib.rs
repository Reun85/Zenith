//! # Vortex
#![feature(optin_builtin_traits)]
// Start of code
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

mod build_constants;
pub mod event;
pub mod infrastructure;
pub mod log;
pub mod undrop;
pub mod window;

pub trait Application
where
    Self: Sized,
{
    type BuildError: std::fmt::Debug;
    type RunError: std::fmt::Debug;

    fn build() -> Result<Self, Self::BuildError>;
    fn run(&mut self) -> Result<(), Self::RunError>;
}

#[derive(Debug, thiserror::Error)]
pub enum UserError<App: Application> {
    #[error("User Application run returned with error {0:?}")]
    RunError(App::RunError),
    #[error("User Application build returned with error {0:?}")]
    BuildError(App::BuildError),
}
#[derive(Debug, thiserror::Error)]
pub enum Error<App: Application> {
    #[error(transparent)]
    External(#[from] UserError<App>),
    #[error(transparent)]
    LoggerError(#[from] log::Error),
}

pub fn start_application<App: Application>() -> Result<(), Error<App>> {
    log::create(log::LoggingCreateInfo {
        level: build_constants::get_logger_level(),
        ..log::LoggingCreateInfo::max()
    })?;
    log::info!("Logging initialized");

    let mut app = {
        let _s = log::trace_span!("Building application");

        match App::build() {
            Ok(x) => x,
            Err(e) => return Err(Error::External(UserError::BuildError(e))),
        }
    };

    {
        let _s = log::trace_span!("Running application");

        match app.run() {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::External(UserError::RunError(e))),
        }
    }
}

type PixelCoordinate = nalgebra::Vector2<u32>;
