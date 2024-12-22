#![allow(dead_code)]
#![allow(unused_variables)]
use std::rc::Rc;
use std::sync::Arc;

use horizon::QueueDescription;

mod winit_app;

pub struct VulkanData {
    instance: Arc<horizon::Instance>,
    surface: Rc<horizon::Surface>,
}

impl VulkanData {
    fn new(window: &winit::window::Window) -> Self {
        // Default info
        let instance = horizon::Instance::new_dynamic(horizon::instance::InstanceCreateInfo {
            application_name: "Test".to_string(),
            ..Default::default()
        })
        .unwrap();

        let surface = instance.create_surface(&window).unwrap();

        let swapchain = horizon::SwapChain::new_promise(horizon::SwapChainDescription {
            surface: surface.clone(),
        });

        let queue = horizon::Queue::new_promise(QueueDescription {
            flags: horizon::QueueFlags::GRAPHICS,
            supports: Some(swapchain.clone()),
            ..QueueDescription::from_count(1)
        });

        let device = horizon::Device::new(
            &instance,
            horizon::DeviceCreationInfo {
                swapchain: vec![swapchain.clone()],
                queues: vec![queue.clone()],
                physical_device_creation_info: Default::default(),
                extensions: vec![],
                layers: vec![],
                order: Default::default(),
            },
        );

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
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_file(true)
            .with_line_number(true)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    )
    .unwrap();
    winit_app::WinitApplication::<App>::run()
}
