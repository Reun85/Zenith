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

impl Drop for Instance {
    fn drop(&mut self) {
        tracing::debug!("Dropping instance");
        unsafe {
            self._debug_utils.destroy_debug_utils_messenger(
                self._debug_messenger,
                self.allocation_callbacks.as_ref(),
            );
        };

        unsafe {
            self.raw
                .destroy_instance(self.allocation_callbacks.as_ref());
        };
    }
}

impl infrastructure::ResourceDeleter<crate::library::Surface> for Instance {
    fn delete(&mut self, resource: &crate::library::Surface) {
        unsafe {
            resource
                .surface_loader
                .destroy_surface(resource.raw, self.allocation_callbacks.as_ref())
        }
    }
}
