mod winit_app;

pub struct App {}

impl winit_app::Application for App {
    fn init(info: winit_app::InitInfo) -> Self {
        let winit_app::InitInfo { window: _window } = info;
        App {}
    }

    fn render(&mut self, info: winit_app::RenderInfo) {
        let winit_app::RenderInfo { window: _window } = info;
    }

    fn window_event(&mut self, info: winit_app::EventInfo) {
        let winit_app::EventInfo { event: _event } = info;
    }

    fn shutdown(&mut self, info: winit_app::ShutdownInfo) {
        let winit_app::ShutdownInfo {} = info;
    }
}

fn main() -> anyhow::Result<()> {
    winit_app::WinitApplication::<App>::run()
}
