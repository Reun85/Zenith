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

#[derive(Debug, Default)]
struct App {}

impl UserApplication for App {
    fn render(&mut self) {
        log::info!("Ran application");
    }
}

fn main() -> anyhow::Result<()> {
    match vortex::start::<App>() {
        Ok(it) => it,
        Err(err) => return Err(err)?,
    };
    Ok(())
}
