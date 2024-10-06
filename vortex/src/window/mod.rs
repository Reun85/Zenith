#[cfg(feature = "use-winit")]
pub mod winit_platform;

#[cfg(feature = "use-winit")]
pub use crate::window::winit_platform as platform_impl;

pub mod input;

pub type AppInitializerFunction =
    dyn Fn(&mut InitContext) -> Result<Box<dyn super::UserApplication>, crate::Error> + 'static;
// TODO: Give this a better name
pub(crate) struct EventLoopInput {
    pub(crate) app: Option<Box<dyn super::UserApplication>>,
    pub(crate) app_creater: Box<AppInitializerFunction>,
}

impl std::fmt::Debug for EventLoopInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLoopInput")
            .field("app", &self.app)
            .finish()
    }
}

#[derive(Debug)]
pub struct Output {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PlatformSpecific(#[from] platform_impl::Error),
}

pub trait InitContextLike {
    /// Create a new window given current context
    fn create_window(&mut self, attributes: WindowAttributes) -> Result<Window, Error>;
}
#[derive(Debug)]
pub struct InitContext<'a>(platform_impl::InitContext<'a>);

impl<'a> InitContextLike for InitContext<'a> {
    fn create_window(&mut self, attributes: WindowAttributes) -> Result<Window, Error> {
        self.0.create_window(attributes)
    }
}

pub(crate) trait EventLoopLike {
    type ApplicationType: ApplicationHandler;

    fn build() -> Self;

    /// The main `EventLoop` of the application resides here
    fn run(self, inp: EventLoopInput) -> Result<Output, Error>;
}

#[derive(Debug)]
pub(crate) struct EventLoop(platform_impl::EventLoop);

impl EventLoopLike for EventLoop {
    type ApplicationType = <platform_impl::EventLoop as EventLoopLike>::ApplicationType;
    fn build() -> Self {
        Self(platform_impl::EventLoop::build())
    }
    fn run(self, inp: EventLoopInput) -> Result<Output, Error> {
        self.0.run(inp)
    }
}

/// Specifically does not require `std::hash::Hash` as most games will have 1 window, using a
/// `HashMap` is overkill
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct WindowID(u64);

#[derive(Debug, derive_more::From)]
pub struct Window(platform_impl::Window);

impl raw_window_handle::HasDisplayHandle for Window {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.0.display_handle()
    }
}

impl raw_window_handle::HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.0.window_handle()
    }
}

impl WindowHandler for Window
where
    Self: raw_window_handle::HasWindowHandle,
{
    fn get_raw_handle(&self) {
        self.0.get_raw_handle()
    }

    fn get_id(&self) -> WindowID {
        self.0.get_id()
    }

    fn inner_size(&self) -> PhysicalSize {
        self.0.inner_size()
    }
}

#[derive(Debug, PartialEq, Eq, derive_more::From)]
pub struct DeviceID(pub(crate) platform_impl::DeviceID);

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

#[derive(Debug, Clone)]
pub struct WindowAttributes {
    pub inner_size: Option<PhysicalSize>,
    pub min_inner_size: Option<PhysicalSize>,
    pub max_inner_size: Option<PhysicalSize>,
    pub position: Option<Position<f64>>,
    pub resizable: bool,
    pub enabled_buttons: WindowButtons,
    pub title: String,
    pub maximized: bool,
    pub visible: bool,
    pub transparent: bool,
    pub blur: bool,
    pub decorations: bool,
    pub window_icon: Option<Icon>,
    pub preferred_theme: Option<Theme>,
    pub resize_increments: Option<PhysicalSize>,
    pub content_protected: bool,
    pub window_level: WindowLevel,
    pub active: bool,
    pub cursor: Cursor,
    pub fullscreen: Option<Fullscreen>,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        WindowAttributes {
            inner_size: None,
            min_inner_size: None,
            max_inner_size: None,
            position: None,
            resizable: true,
            enabled_buttons: WindowButtons::all(),
            title: "winit window".to_owned(),
            maximized: false,
            fullscreen: None,
            visible: true,
            transparent: false,
            blur: false,
            decorations: true,
            window_level: Default::default(),
            window_icon: None,
            preferred_theme: None,
            resize_increments: None,
            content_protected: false,
            cursor: Cursor::default(),
            active: true,
        }
    }
}

pub trait WindowHandler {
    fn get_raw_handle(&self);
    fn get_id(&self) -> WindowID;
    fn inner_size(&self) -> PhysicalSize;
}

pub trait ApplicationHandler {}

/// Size of a window stored in pixels
#[derive(Debug, Clone, Copy)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize> for nalgebra::Vector2<u32> {
    fn from(val: PhysicalSize) -> Self {
        nalgebra::Vector2::new(val.width, val.height)
    }
}
impl From<nalgebra::Vector2<u32>> for PhysicalSize {
    fn from(v: nalgebra::Vector2<u32>) -> Self {
        PhysicalSize {
            width: v.x,
            height: v.y,
        }
    }
}
/// Offset on monitor in pixels
#[derive(Debug, Clone, Copy)]
pub struct Position<T> {
    x: T,
    y: T,
}

impl<T: Copy> From<Position<T>> for nalgebra::Vector2<T> {
    fn from(val: Position<T>) -> Self {
        nalgebra::Vector2::new(val.x, val.y)
    }
}

impl<T> From<nalgebra::Vector2<T>> for Position<T>
where
    T: Copy + std::fmt::Debug + std::cmp::PartialEq + 'static,
{
    fn from(v: nalgebra::Vector2<T>) -> Self {
        Position { x: v.x, y: v.y }
    }
}
