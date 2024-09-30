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
impl crate::infrastructure::StateConstains for EventCategories {
    fn contains(&self, other: &Self) -> bool {
        self.contains(*other)
    }
}

use crate::event::{EventLike, HasStaticCategory};

pub mod keyboard {
    use super::{EventCategories, EventLike, HasStaticCategory};

    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum Event {
        KeyPress(Press),
        KeyRelease(Release),
    }
    #[derive(Debug, derive_more::From)]
    pub struct Key {}
    #[derive(Debug, derive_more::From)]
    pub struct Press {
        pub device_id: crate::window::DeviceID,
        pub key: Key,
        pub repeat: bool,
    }

    #[derive(Debug, derive_more::From)]
    pub struct Release {
        pub device_id: crate::window::DeviceID,
        pub key: Key,
    }

    impl HasStaticCategory for Event {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndKeyboard;
    }
    impl HasStaticCategory for Press {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndKeyboard;
    }
    impl HasStaticCategory for Release {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndKeyboard;
    }
}
pub mod mouse {
    use super::{EventCategories, EventLike, HasStaticCategory};
    #[derive(Debug, derive_more::From)]
    pub enum Button {
        Left,
        Right,
        Middle,
        Back,
        Forward,
        Other(u16),
    }
    #[derive(Debug, derive_more::From, derive_more::IsVariant, derive_more::TryInto)]
    pub enum Event {
        MousePress(Press),
        MouseRelease(Release),
        MouseMove(Move),
    }
    impl HasStaticCategory for Event {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }
    #[derive(Debug, derive_more::From)]

    pub struct Release {
        pub device_id: crate::window::DeviceID,
        pub button: Button,
    }
    impl HasStaticCategory for Release {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }

    #[derive(Debug, derive_more::From)]
    pub struct Press {
        pub device_id: crate::window::DeviceID,
        pub button: Button,
        pub repeat: bool,
    }
    impl HasStaticCategory for Press {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::InputAndMouse;
    }

    #[derive(Debug, derive_more::From)]
    pub struct Move {
        pub device_id: crate::window::DeviceID,
        pub position: crate::window::Position,
    }
    impl HasStaticCategory for Move {
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
    pub struct CloseEvent {
        pub id: crate::window::WindowID,
    }
    impl HasStaticCategory for CloseEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::Window;
    }
    #[derive(Debug, derive_more::From)]
    pub struct ResizeEvent {
        pub id: crate::window::WindowID,
        pub size: crate::window::Size,
    }
    impl HasStaticCategory for ResizeEvent {
        type Category = EventCategories;
        const CATEGORY: EventCategories = EventCategories::Window;
    }
}

impl EventLike for Event {
    type Category = EventCategories;
    fn get_category(&self) -> EventCategories {
        match self {
            Event::KeyBoard(x) => x.get_category(),
            Event::Mouse(x) => x.get_category(),
            Event::Window(x) => x.get_category(),
        }
    }
}
