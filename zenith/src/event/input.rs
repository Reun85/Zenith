#![allow(unused_imports)]

#[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
pub enum Event {
    KeyBoard(keyboard::Event),
    Mouse(mouse::Event),
    Window(window::Event),
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq,Eq, PartialOrd, Ord,Hash)]
    pub struct EventCategories: u32 {
        const None= 1u32<<0;
        const Input = 1u32 << 1;
        const Keyboard = 1u32<<2;
        const Mouse = 1u32<<3;
        const Window = 1u32<<4;


        const InputAndKeyboard = Self::Input.bits() | Self::Keyboard.bits();
        const InputAndMouse = Self::Input.bits() | Self::Mouse.bits();
    }
}

use super::EventLike;
use super::HasStaticCategory;

pub mod keyboard {
    use super::{EventCategories, EventLike, HasStaticCategory};

    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum Event {
        KeyPress(PressEvent),
        KeyRelease(ReleaseEvent),
    }
    #[derive(Debug, derive_more::From)]
    pub struct Key {}
    #[derive(Debug, derive_more::From)]
    pub struct PressEvent {
        pub key: Key,
        pub repeat: bool,
    }

    #[derive(Debug, derive_more::From)]
    pub struct ReleaseEvent {
        pub key: Key,
    }

    impl HasStaticCategory for Event {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndKeyboard;
    }
    impl HasStaticCategory for PressEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndKeyboard;
    }
    impl HasStaticCategory for ReleaseEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndKeyboard;
    }
}
pub mod mouse {
    use super::{EventCategories, EventLike, HasStaticCategory};
    #[derive(Debug, derive_more::From)]
    pub struct Button {}
    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum Event {
        MousePress(PressEvent),
        MouseRelease(ReleaseEvent),
        MouseMove(MoveEvent),
    }
    impl HasStaticCategory for Event {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }
    #[derive(Debug, derive_more::From)]

    pub struct ReleaseEvent {
        pub button: Button,
    }
    impl HasStaticCategory for ReleaseEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }

    #[derive(Debug, derive_more::From)]
    pub struct PressEvent {
        pub button: Button,
        pub repeat: bool,
    }
    impl HasStaticCategory for PressEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }

    #[derive(Debug, derive_more::From)]
    pub struct MoveEvent {
        pub position: crate::PixelCoordinate,
    }
    impl HasStaticCategory for MoveEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }
}

pub mod window {
    use super::{EventCategories, HasStaticCategory};
    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum Event {
        WindowClose(CloseEvent),
        WindowResize(ResizeEvent),
    }
    impl HasStaticCategory for Event {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::Window;
    }
    #[derive(Debug, derive_more::From)]
    pub struct CloseEvent {}
    impl HasStaticCategory for CloseEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::Window;
    }
    #[derive(Debug, derive_more::From)]
    pub struct ResizeEvent {
        pub size: crate::PixelCoordinate,
    }
    impl HasStaticCategory for ResizeEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::Window;
    }
}

impl super::EventLike for Event {
    type Category = EventCategories;
    fn get_category(&self) -> EventCategories {
        match self {
            Event::KeyBoard(x) => x.get_category(),
            Event::Mouse(x) => x.get_category(),
            Event::Window(x) => x.get_category(),
        }
    }
}
