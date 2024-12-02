#[derive(derive_more::Deref)]
pub struct Surface {
    #[deref]
    pub raw: ash::vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
}
