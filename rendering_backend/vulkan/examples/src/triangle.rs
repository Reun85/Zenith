use std::sync::Arc;

mod winit_app;

pub struct VulkanData {
    instance: Arc<horizon::Instance>,
    surface: horizon::Surface,
}

impl VulkanData {
    fn new(window: &winit::window::Window) -> Self {
        let instance = horizon::Instance::new_linked(horizon::instance::InstanceCreateInfo {
            application_name: "Test".to_string(),
            ..Default::default()
        })
        .unwrap();
        let surface = instance.create_surface(&window).unwrap();
        Self { instance, surface }
    }
}

pub struct App {
    vulkan_data: VulkanData,
}

impl winit_app::Application for App {
    fn init(info: winit_app::InitInfo) -> Self {
        let winit_app::InitInfo { window } = info;

        App {
            vulkan_data: VulkanData::new(window),
        }
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
