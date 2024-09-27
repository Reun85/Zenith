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
    fn set_flags(&self, flags: &Self) -> Self;
    /// Should be used for single flag
    fn rem_flags(&self, flags: &Self) -> Self;
}

pub trait StateConstains {
    fn contains(&self, other: &Self) -> bool;
}

pub type PixelCoordinate = nalgebra::Vector2<u32>;
