/// Default constraints for types
// Will include the Zenith specific viewer later
pub trait Debug
where
    Self: std::fmt::Debug,
{
}

impl<T> Debug for T where T: std::fmt::Debug {}

pub trait Flags
where
    Self: Debug,
{
    /// Should be used for single flag
    fn has_flags(&self, flags: &Self) -> bool;
    /// Should be used for single flag
    fn set_flags(&mut self, flags: &Self);
    /// Should be used for multi state flags
    fn intersects(&self, flags: &Self) -> bool;
}

pub trait StateConstains {
    fn contains(&self, other: &Self) -> bool;
}

impl<T: Flags> StateConstains for T {
    fn contains(&self, other: &Self) -> bool {
        self.has_flags(other)
    }
}
