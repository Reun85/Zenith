pub mod input;

/// Marker Trait
pub trait EventCategory
where
    Self: crate::infrastructure::Debug + Clone + Hash,
{
    fn contains(&self, other: &Self) -> bool;
}

// TODO: Change this to own Flags impl trait
impl EventCategory for () {
    fn contains(&self, other: &Self) -> bool {
        true
    }
}
impl<T> EventCategory for T
where
    T: crate::infrastructure::StateConstains + crate::infrastructure::Debug,
{
    fn contains(&self, other: &Self) -> bool {
        crate::infrastructure::StateConstains::contains(self, other)
    }
}

/// Marker Trait for data types that may be dispatched using  `EventDispatcher`
pub trait EventLike
where
    Self: crate::infrastructure::Debug,
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
    fn new(event: E) -> Self {
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
