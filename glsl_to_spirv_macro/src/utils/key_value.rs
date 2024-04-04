use std::ops::{Deref, DerefMut};

pub struct KeyValue<K, V> {
    pub key: K,
    pub value: V,
}
impl<K, V> KeyValue<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}
impl<K, V> DerefMut for KeyValue<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
impl<K, V> Deref for KeyValue<K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
