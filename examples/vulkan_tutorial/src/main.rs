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
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.renderer.run()
    }
}

fn main() {
    log::create(log::LoggingCreateInfo {
        level: log::Level::TRACE,
        ..log::LoggingCreateInfo::max()
    });

    let res = run_program();
    match res {
        Ok(x) => log::info!("Application shutdown successfully! Result: {:?}", x),
        Err(e) => log::error!("Fatal error occurred: {}", e),
    }
}
fn run_program() -> anyhow::Result<()> {
    let mut app = App::new()?;

    app.run()?;
    log::debug!("App has shut down...");
    Ok(())
}
