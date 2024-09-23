use zenith::log;

mod vulkan;
struct App {
    renderer: vulkan::VulkanApp,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let renderer = vulkan::VulkanApp::new()?;
        Ok(Self { renderer })
    }
    pub fn run(&mut self) -> anyhow::Result<vulkan::RunReturnType> {
        self.renderer.run()
    }
}

fn start() -> anyhow::Result<()> {
    log::create(log::LoggingCreateInfo {
        level: log::Level::TRACE,
        ..log::LoggingCreateInfo::max()
    });
    match run_program() {
        Ok(x) => {
            log::info!("Application shutdown successfully! Result: {:?}", x);
            Ok(())
        }
        Err(e) => {
            log::error!("Fatal error occurred: {}", e);
            Err(e)
        }
    }
}
fn run_program() -> anyhow::Result<vulkan::RunReturnType> {
    let mut app = App::new()?;
    app.run()
}

fn main() -> anyhow::Result<()> {
    start()
}
