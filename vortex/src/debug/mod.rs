pub trait Debug
where
    Self: std::fmt::Debug,
{
}

impl<T> Debug for T where T: std::fmt::Debug {}
