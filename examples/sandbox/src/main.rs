#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

use zenith::log;
use zenith::Application;

#[derive(Debug)]
struct App {}

impl Application for App {
    type BuildError = ();

    type RunError = ();

    fn build() -> Result<Self, Self::BuildError> {
        log::info!("Building application");
        Ok(Self {})
    }

    fn run(&mut self) -> Result<(), Self::RunError> {
        log::info!("Ran application");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    zenith::start_application::<App>()?;
    Ok(())
}
