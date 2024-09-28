//! https://rust-windowing.github.io/winit/winit/index.html

pub use winit::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error creating window {0:?}")]
    EventLoopError(#[from] winit::error::EventLoopError),
}

#[derive(Debug, smart_default::SmartDefault)]
enum State {
    #[default]
    Uninit,
    Init,
    Running,
    Suspended,
    Exited,
}

#[derive(Debug)]
pub(crate) struct WinitApplication {
    /// There is a chance there is a need for more than one window. It is highly unlikely, but we
    /// have to account for it
    windows: smallvec::SmallVec<[Window; 1]>,
    app: super::EventLoopInput,
    state: State,
}
impl super::ApplicationHandler for WinitApplication {}

impl winit::application::ApplicationHandler for WinitApplication {
    fn new_events(&mut self, event_loop: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        let _ = (event_loop, cause);
        match cause {
            event::StartCause::ResumeTimeReached {
                start: _,
                requested_resume: _,
            } => {
                self.state = State::Running;
            }
            event::StartCause::WaitCancelled {
                start: _,
                requested_resume: _,
            } => {
                self.state = State::Running;
            }
            event::StartCause::Poll => {
                self.state = State::Init;
            }
            event::StartCause::Init => {
                self.state = State::Init;
            }
        }
    }

    fn user_event(&mut self, event_loop: &event_loop::ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        device_id: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
        self.state = State::Suspended;
    }

    fn exiting(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
        self.state = State::Exited;
    }

    fn memory_warning(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        match self.state {
            State::Uninit => todo!(),
            State::Init => {
                self.windows.push(
                    event_loop
                        .create_window(winit::window::WindowAttributes::default())
                        .unwrap()
                        .into(),
                );
                self.state = State::Running;
            }
            State::Running => todo!(),
            State::Suspended => todo!(),
            State::Exited => todo!(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: event::WindowEvent,
    ) {
        use winit::event::*;
        let window_id: super::WindowID = window_id.into();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.windows[0].as_ref().request_redraw();
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Context {
    event_loop: event_loop::EventLoop<()>,
}

impl super::PlatformlessContext for Context {
    type WindowType = Window;
    type ApplicationType = WinitApplication;

    fn run(self, inp: super::EventLoopInput) -> Result<super::Output, super::Error> {
        use winit::event_loop::*;
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        self.event_loop.set_control_flow(ControlFlow::Poll);
        // // ControlFlow::Wait pauses the event loop if no events are available to process.
        // // This is ideal for non-game applications that only update in response to user
        // // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        // self.event_loop.set_control_flow(ControlFlow::Wait);
        let mut app = WinitApplication {
            windows: smallvec::SmallVec::new(),
            app: inp,
            state: State::Uninit,
        };
        if let Err(e) = self.event_loop.run_app(&mut app) {
            return Err(Error::EventLoopError(e).into());
        }
        Ok(super::Output {})
    }

    fn build() -> Self {
        use winit::event_loop::*;
        let event_loop = EventLoop::new().unwrap();
        Self { event_loop }
    }
}

#[derive(Debug, derive_more::Deref)]
pub(crate) struct Window {
    window: Box<winit::window::Window>,
}

impl From<winit::window::Window> for Window {
    fn from(window: winit::window::Window) -> Self {
        Self {
            window: Box::new(window),
        }
    }
}

impl Into<super::WindowID> for winit::window::WindowId {
    fn into(self) -> super::WindowID {
        super::WindowID(self.into())
    }
}

impl super::WindowHandler for Window {
    fn get_raw_handle(&self) {
        todo!()
    }
    fn get_id(&self) -> super::WindowID {
        self.window.id().into()
    }
}

impl Into<winit::window::WindowAttributes> for super::WindowAttributes {
    fn into(self) -> winit::window::WindowAttributes {
        winit::window::WindowAttributes {
            title: self.title,
            transparent: self.transparent,
            decorations: self.decorations,
            inner_size: self.inner_size.map(|size| size.into()),
            min_inner_size: self.min_inner_size.map(|size| size.into()),
            max_inner_size: self.max_inner_size.map(|size| size.into()),
            resizable: self.resizable,
            maximized: self.maximized,
            visible: self.visible,
            position: self.position.map(|pos| pos.into()),
            fullscreen: self.fullscreen.map(|fullscreen| fullscreen.into()),
        }
    }
}

impl Into<winit::dpi::Size> for super::Size {
    fn into(self: super::Size) -> winit::dpi::Size {
        winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(self.x, self.y))
    }
}
/// Size of a window stored in pixels
pub type Size = nalgebra::Vector2<u32>;
/// Offset on monitor in pixels
pub type Position = nalgebra::Vector2<u32>;

impl Into<winit::window::Fullscreen> for super::Fullscreen {
    fn into(self) -> winit::window::Fullscreen {
        match self {
            super::Fullscreen::Exclusive => winit::window::Fullscreen::Exclusive,
            super::Fullscreen::Borderless => winit::window::Fullscreen::Borderless(None),
        }
    }
}
bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq,Eq, PartialOrd, Ord,Hash)]
    pub struct WindowButtons: u8 {
        const None= 1u8<<0;
        const Close = 1u8 << 1;
        const Minimize = 1u8<<2;
        const Maximize = 1u8<<3;
    }
}

#[derive(Debug, Clone, Copy, smart_default::SmartDefault)]
pub enum WindowLevel {
    /// The window will always be below normal windows.
    ///
    /// This is useful for a widget-based app.
    AlwaysOnBottom,

    /// The default.
    #[default]
    Normal,

    /// The window will always be on top of normal windows.
    AlwaysOnTop,
}

#[derive(Debug, Clone, Copy, smart_default::SmartDefault)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

#[derive(Debug, Clone, Copy, smart_default::SmartDefault)]
pub struct Icon {}
#[derive(Debug, Clone, Copy, smart_default::SmartDefault)]
pub enum Cursor {
    #[default]
    Builtin(CursorIcon),
    Custom,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CursorIcon {
    /// The platform-dependent default cursor. Often rendered as arrow.
    #[default]
    Default,

    /// A context menu is available for the object under the cursor. Often
    /// rendered as an arrow with a small menu-like graphic next to it.
    ContextMenu,

    /// Help is available for the object under the cursor. Often rendered as a
    /// question mark or a balloon.
    Help,

    /// The cursor is a pointer that indicates a link. Often rendered as the
    /// backside of a hand with the index finger extended.
    Pointer,

    /// A progress indicator. The program is performing some processing, but is
    /// different from [`CursorIcon::Wait`] in that the user may still interact
    /// with the program.
    Progress,

    /// Indicates that the program is busy and the user should wait. Often
    /// rendered as a watch or hourglass.
    Wait,

    /// Indicates that a cell or set of cells may be selected. Often rendered as
    /// a thick plus-sign with a dot in the middle.
    Cell,

    /// A simple crosshair (e.g., short line segments resembling a "+" sign).
    /// Often used to indicate a two dimensional bitmap selection mode.
    Crosshair,

    /// Indicates text that may be selected. Often rendered as an I-beam.
    Text,

    /// Indicates vertical-text that may be selected. Often rendered as a
    /// horizontal I-beam.
    VerticalText,

    /// Indicates an alias of/shortcut to something is to be created. Often
    /// rendered as an arrow with a small curved arrow next to it.
    Alias,

    /// Indicates something is to be copied. Often rendered as an arrow with a
    /// small plus sign next to it.
    Copy,

    /// Indicates something is to be moved.
    Move,

    /// Indicates that the dragged item cannot be dropped at the current cursor
    /// location. Often rendered as a hand or pointer with a small circle with a
    /// line through it.
    NoDrop,

    /// Indicates that the requested action will not be carried out. Often
    /// rendered as a circle with a line through it.
    NotAllowed,

    /// Indicates that something can be grabbed (dragged to be moved). Often
    /// rendered as the backside of an open hand.
    Grab,

    /// Indicates that something is being grabbed (dragged to be moved). Often
    /// rendered as the backside of a hand with fingers closed mostly out of
    /// view.
    Grabbing,

    /// The east border to be moved.
    EResize,

    /// The north border to be moved.
    NResize,

    /// The north-east corner to be moved.
    NeResize,

    /// The north-west corner to be moved.
    NwResize,

    /// The south border to be moved.
    SResize,

    /// The south-east corner to be moved.
    SeResize,

    /// The south-west corner to be moved.
    SwResize,

    /// The west border to be moved.
    WResize,

    /// The east and west borders to be moved.
    EwResize,

    /// The south and north borders to be moved.
    NsResize,

    /// The north-east and south-west corners to be moved.
    NeswResize,

    /// The north-west and south-east corners to be moved.
    NwseResize,

    /// Indicates that the item/column can be resized horizontally. Often
    /// rendered as arrows pointing left and right with a vertical bar
    /// separating them.
    ColResize,

    /// Indicates that the item/row can be resized vertically. Often rendered as
    /// arrows pointing up and down with a horizontal bar separating them.
    RowResize,

    /// Indicates that the something can be scrolled in any direction. Often
    /// rendered as arrows pointing up, down, left, and right with a dot in the
    /// middle.
    AllScroll,

    /// Indicates that something can be zoomed in. Often rendered as a
    /// magnifying glass with a "+" in the center of the glass.
    ZoomIn,

    /// Indicates that something can be zoomed in. Often rendered as a
    /// magnifying glass with a "-" in the center of the glass.
    ZoomOut,
}

/// Fullscreen modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fullscreen {
    Exclusive,

    /// Providing `None` to `Borderless` will fullscreen on the current monitor.
    Borderless,
}
