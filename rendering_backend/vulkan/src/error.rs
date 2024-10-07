#[derive(Debug, thiserror::Error)]
pub enum Generic {
    #[error(transparent)]
    Vk(#[from] VkError),
    // These values are cloned
    #[error("Missing required extensions {0:?}")]
    RequiredExtensionsMissing(Vec<super::types::ExtensionName>),
    #[error(transparent)]
    Library(#[from] ash::LoadingError),
    #[error("Failed to get window handle {0}")]
    HandleError(#[from] raw_window_handle::HandleError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct VkError(#[from] ash::vk::Result);
