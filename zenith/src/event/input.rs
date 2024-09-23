#[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
pub enum Event {
    KeyBoard(keyboard::KeyBoardEvent),
    Mouse(mouse::MouseEvent),
    Window(window::WindowEvent),
}

pub mod keyboard {

    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum KeyBoardEvent {
        KeyPress(KeyPressEvent),
        KeyRelease(KeyReleaseEvent),
    }
    #[derive(Debug, derive_more::From)]
    pub struct Key {}
    #[derive(Debug, derive_more::From)]
    pub struct KeyPressEvent {
        pub key: Key,
        pub repeat: bool,
    }

    #[derive(Debug, derive_more::From)]
    pub struct KeyReleaseEvent {
        pub key: Key,
    }
}
pub mod mouse {
    #[derive(Debug, derive_more::From)]
    pub struct Button {}
    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum MouseEvent {
        MousePress(MousePressEvent),
        MouseRelease(MouseReleaseEvent),
        MouseMove(MouseMoveEvent),
    }
    #[derive(Debug, derive_more::From)]

    pub struct MouseReleaseEvent {
        pub button: Button,
    }

    #[derive(Debug, derive_more::From)]
    pub struct MousePressEvent {
        pub button: Button,
        pub repeat: bool,
    }

    #[derive(Debug, derive_more::From)]
    pub struct MouseMoveEvent {
        pub position: crate::PixelCoordinate,
    }
}

pub mod window {
    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum WindowEvent {
        WindowClose(WindowCloseEvent),
        WindowResize(WindowResizeEvent),
    }
    #[derive(Debug, derive_more::From)]
    pub struct WindowCloseEvent {}
    #[derive(Debug, derive_more::From)]
    pub struct WindowResizeEvent {
        pub size: crate::PixelCoordinate,
    }
}

impl super::EventLike for Event {}
