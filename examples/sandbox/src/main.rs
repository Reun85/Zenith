#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

use vortex::log;
use vortex::UserApplication;

#[derive(Debug)]
struct App {}

impl UserApplication for App {
    type BuildError = ();

    fn new() -> Result<Self, Self::BuildError> {
        log::info!("Building application");
        Ok(Self {})
    }

    fn render(&mut self) {
        log::info!("Ran application");
    }
}

fn main() -> anyhow::Result<()> {
    vortex::start_application::<App>()?;
    Ok(())
}
