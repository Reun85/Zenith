#[derive(Debug, thiserror::Error)]
pub enum Error {
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
}

impl From<ash::vk::Result> for Error {
    fn from(e: ash::vk::Result) -> Self {
        Error::Vk(VkError(e))
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct VkError(#[from] ash::vk::Result);
