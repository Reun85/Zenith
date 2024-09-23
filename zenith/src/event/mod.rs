pub mod input;

/// Marker Trait for data types that may be dispatch using  `EventDispatcher`
pub trait EventLike
where
    Self: Sized + std::fmt::Debug,
{
}

/// Marker Trait for data types that may can be a callback for `EventDispatcher`
pub trait EventCallbackLike<E: EventLike> {
    fn call(&self, event: &E);
}

pub trait EventDispatcher {
    fn dispatch<E: EventLike>(&self, callback: impl EventCallbackLike<E>);
}
