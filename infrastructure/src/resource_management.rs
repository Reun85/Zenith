pub trait ResourceDeleter<Resource> {
    fn delete(&mut self, resource: &mut Resource);
}
impl<Resource, Owner> ResourceDeleter<Resource> for Owner
where
    Owner: AsMut<dyn ResourceDeleter<Resource>>,
{
    fn delete(&mut self, resource: &mut Resource) {
        self.as_mut().delete(resource);
    }
}

/// Used for a owning resourceReference where deleting the resource requires larger context, that
/// owner provides
#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct ResourceRef<Value, Owner: ResourceDeleter<Value>> {
    #[deref]
    #[deref_mut]
    pub value: Value,
    pub owner: Owner,
}

impl<Value, Owner: ResourceDeleter<Value>> Drop for ResourceRef<Value, Owner> {
    fn drop(&mut self) {
        self.owner.delete(&mut self.value);
    }
}
