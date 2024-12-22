#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error(transparent)]
    Vk(#[from] VkError),
    #[error("Driver could not enumerate physical deives")]
    EnumeratePhysicalDevicesFailed,
    #[error("No physical device match the requirements")]
    SuitablePhysicalDeviceNotFound,

    // These values are cloned
    #[error("Missing required extensions {0:?}")]
    RequiredExtensionsMissing(Vec<super::types::ExtensionName>),
    #[error(transparent)]
    Library(#[from] ash::LoadingError),
    #[error("Failed to get window handle {0}")]
    HandleError(#[from] raw_window_handle::HandleError),
    #[error("Queue description could not be fit: {0:?}")]
    QueueDescriptionCouldNotBeFilled(super::device::QueueDescription),
    #[error("Swapchain could not be created {0:?}")]
    SwapchainCreationFailed(VkError),
}

impl From<ash::vk::Result> for InitError {
    fn from(e: ash::vk::Result) -> Self {
        InitError::Vk(VkError(e))
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct VkError(#[from] ash::vk::Result);
