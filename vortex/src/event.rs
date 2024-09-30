/// Marker Trait
#[allow(clippy::module_name_repetitions)]
pub trait EventCategory
where
    Self: crate::infrastructure::VortexDebug + Clone + std::hash::Hash,
{
    fn contains(&self, other: &Self) -> bool;
}

impl EventCategory for () {
    fn contains(&self, (): &Self) -> bool {
        true
    }
}

impl<T> EventCategory for T
where
    T: Clone
        + std::hash::Hash
        + crate::infrastructure::StateConstains
        + crate::infrastructure::VortexDebug,
{
    fn contains(&self, other: &Self) -> bool {
        crate::infrastructure::StateConstains::contains(self, other)
    }
}

/// Marker Trait for data types that may be dispatched using  `EventDispatcher`
#[allow(clippy::module_name_repetitions)]
pub trait EventLike
where
    Self: crate::infrastructure::VortexDebug,
{
    /// The `EventCategory` this Event belongs to
    /// set to () if you don't wish to use `EventCategory`-s
    type Category: EventCategory;
    fn get_category(&self) -> Self::Category;
}

pub trait HasStaticCategory
where
    Self: std::fmt::Debug,
{
    type Category: EventCategory;
    const CATEGORY: Self::Category;
}

impl<T: HasStaticCategory> EventLike for T {
    type Category = T::Category;

    fn get_category(&self) -> Self::Category {
        T::CATEGORY
    }
}

/// Marker Trait for data types that may can be a callback for `EventDispatcher`
#[allow(clippy::module_name_repetitions)]
pub trait EventCallbackLike<E: EventLike> {
    /// Returns whether the event was handled
    fn call(&self, event: &E) -> bool;
}

impl<E, F> EventCallbackLike<E> for F
where
    E: EventLike,
    F: Fn(&E) -> bool,
{
    fn call(&self, event: &E) -> bool {
        self(event)
    }
}

pub trait Dispatcher<E: EventLike> {
    fn dispatch<F: HasStaticCategory<Category = E::Category>>(
        &mut self,
        callback: impl EventCallbackLike<E>,
    );
}

#[derive(Debug, derive_more::From)]
pub struct BasicDispatcher<E>
where
    E: EventLike,
{
    event: E,
    handled: bool,
}

impl<E: EventLike> BasicDispatcher<E> {
    const fn new(event: E) -> Self {
        Self {
            event,
            handled: false,
        }
    }
}

impl<E: EventLike> Dispatcher<E> for BasicDispatcher<E> {
    fn dispatch<F: HasStaticCategory<Category = E::Category>>(
        &mut self,
        callback: impl EventCallbackLike<E>,
    ) {
        if !self.handled {
            return;
        }
        let category = self.event.get_category();
        if category.contains(&F::CATEGORY) {
            self.handled = callback.call(&self.event);
        }
    }
}