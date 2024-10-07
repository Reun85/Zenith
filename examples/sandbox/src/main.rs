#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]
use std::num::NonZeroU32;
use std::rc::Rc;
use vortex::entry::UserApplication;
use vortex::log;
use vortex::window::{InitContextLike, WindowHandler};

struct App {
    window: Rc<vortex::window::Window>,

    context: softbuffer::Context<Rc<vortex::window::Window>>,
    surface: softbuffer::Surface<Rc<vortex::window::Window>, Rc<vortex::window::Window>>,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "App {{ window: {:?} }}", self.window)
    }
}

impl UserApplication for App {
    fn render(&mut self) {
        log::info_span!("Ran application");
        let (width, height) = {
            let size = self.window.inner_size();
            (size.width, size.height)
        };
        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        let mut buffer = self.surface.buffer_mut().unwrap();
        for index in 0..(width * height) {
            let y = index / width;
            let x = index % width;
            let red = x % 255;
            let green = y % 255;
            let blue = (x * y) % 255;

            buffer[index as usize] = blue | (green << 8) | (red << 16);
        }

        buffer.present().unwrap();
    }

    fn update(&mut self) {}

    fn on_window_event(
        &mut self,
        event: &dyn vortex::event::EventLike<Category = vortex::window::input::EventCategories>,
    ) {
        log::debug!("Event: {:?}", event);
        let _ = event;
    }

    fn on_exit(&mut self) {}
}

impl vortex::entry::UserApplicationBuilder for App {
    type Output = Self;
    fn new(
        context: &mut vortex::window::InitContext,
    ) -> Result<Self::Output, vortex::entry::Error> {
        let window = Rc::new(context.create_window(vortex::window::WindowAttributes::default())?);

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        Ok(Self {
            window,
            context,
            surface,
        })
    }
}

fn main() -> anyhow::Result<()> {
    match vortex::entry::start::<App>() {
        Ok(it) => it,
        Err(err) => return Err(err)?,
    };
    Ok(())
}
