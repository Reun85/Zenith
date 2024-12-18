use crate::event::EventLike;
use crate::window;
use crate::window::EventLoopLike;
// If the result is an Err its fine to use box as this will definitely lead to a shutdown.
pub trait UserApplication
where
    Self: crate::debug::Debug,
{
    /// Run per game update
    /// Currently unused
    fn update(&mut self) {}

    /// Run per each frame update
    fn render(&mut self) {}

    fn on_window_event(
        &mut self,
        event: &dyn EventLike<Category = window::input::EventCategories>,
    ) {
        let _ = event;
    }

    /// This function is always preceded by [`WindowCloseEvents`](`event::WindowCloseEvent`) to
    /// [`on_window_event`](`Self::on_window_event`)
    fn on_exit(&mut self) {}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    LoggerError(#[from] crate::log::Error),
    #[error(transparent)]
    WindowManager(#[from] window::Error),
}

pub trait UserApplicationBuilder {
    type Output: UserApplication;
    fn new(context: &mut window::InitContext) -> Result<Self::Output, Error>;
}

/// # Errors
/// Will return an error if logger fails to initialize or the application propagates an error back
pub fn start<AppBuilder: UserApplicationBuilder>() -> Result<window::Output, Error>
where
    <AppBuilder as UserApplicationBuilder>::Output: 'static,
{
    crate::log::init_logging()?;

    {
        let window_context = <window::EventLoop as window::EventLoopLike>::build();
        let f = |context: &mut window::InitContext| {
            AppBuilder::new(context).map(|app| Box::new(app) as Box<dyn UserApplication>)
        };
        let ev_inp = window::EventLoopInput {
            app: None,
            app_creater: Box::new(f),
        };
        let output = window_context.run(ev_inp)?;
        Ok(output)
    }
}
