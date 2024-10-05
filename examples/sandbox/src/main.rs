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

#[derive(Debug)]
struct App {
    window: vortex::window::Window,
}

impl UserApplication for App {
    fn render(&mut self) {
        log::info!("Ran application");
    }

    fn update(&mut self) {}

    fn on_window_event(
        &mut self,
        event: &dyn vortex::event::EventLike<Category = vortex::window::input::EventCategories>,
    ) {
        let _ = event;
    }

    fn on_exit(&mut self) {}
}

impl vortex::UserApplicationBuilder for App {
    type Output = Self;
    fn new(context: &mut vortex::window::InitContext) -> Result<Self::Output, vortex::Error> {
        let window = context.create_window(vortex::window::WindowAttributes::default())?;
        Ok(Self { window })
    }
}

fn main() -> anyhow::Result<()> {
    match vortex::start::<App>() {
        Ok(it) => it,
        Err(err) => return Err(err)?,
    };
    Ok(())
}
