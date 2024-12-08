#[derive(derive_more::Deref)]
pub struct Surface {
    #[deref]
    pub raw: ash::vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
}

// TODO: figure out a better Debug impl
impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface").finish()
    }
}
