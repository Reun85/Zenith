pub struct Instance {
    pub(super) raw: ash::Instance,
    pub allocation_callbacks: Option<ash::vk::AllocationCallbacks<'static>>,
    pub app_name: std::ffi::CString,
    pub(super) _debug_utils: ash::ext::debug_utils::Instance,
    pub(super) _debug_messenger: ash::vk::DebugUtilsMessengerEXT,
    pub callback_data: Box<super::library::DebugCallBackData>,
    // _debug_utils: ash::extensions::ext::DebugUtils,
    // _debug_messenger: ash::vk::DebugUtilsMessengerEXT,
}
