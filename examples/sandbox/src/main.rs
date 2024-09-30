#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

use vortex::log;
use vortex::window::InitContextLike;
use vortex::UserApplication;

#[derive(Debug, Default)]
struct App {
    windows: Option<vortex::window::Window>,
}

impl UserApplication for App {
    fn render(&mut self) {
        log::info!("Ran application");
    }

    fn update(&mut self) {}

    fn on_init(
        &mut self,
        context: &mut vortex::window::InitContext,
    ) -> Result<(), vortex::window::Error> {
        self.windows = Some(context.create_window(vortex::window::WindowAttributes::default())?);
        Ok(())
    }

    fn on_window_event(
        &mut self,
        event: &dyn vortex::event::EventLike<Category = vortex::window::input::EventCategories>,
    ) {
        let _ = event;
    }

    fn on_exit(&mut self) {}
}

fn main() -> anyhow::Result<()> {
    match vortex::start::<App>() {
        Ok(it) => it,
        Err(err) => return Err(err)?,
    };
    Ok(())
}
