use super::error;
use super::raw;
use super::types::{ExtensionName, ExtensionProperties, Layer};
use std::sync::Arc;
use std::{ffi::CStr, ops::Deref};

#[derive(derive_more::Deref)]
pub struct Surface {
    #[deref]
    pub raw: ash::vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
}

// TODO: figure out a better Debug impl
impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface").field("raw", &self.raw).finish()
    }
}
#[cfg(not(build_type = "dist"))]
pub struct DebugInstance {
    pub(crate) _debug_utils: ash::ext::debug_utils::Instance,
    pub(crate) _debug_messenger: ash::vk::DebugUtilsMessengerEXT,
    pub(crate) _callback_data: Box<DebugCallBackData>,
    allocation_callbacks: Option<std::rc::Rc<ash::vk::AllocationCallbacks<'static>>>,
}
pub struct Instance {
    pub(super) raw: ash::Instance,
    pub allocation_callbacks: Option<std::rc::Rc<ash::vk::AllocationCallbacks<'static>>>,
    pub app_name: std::ffi::CString,
    #[cfg(not(build_type = "dist"))]
    pub(crate) debug_messenger_instance: Option<DebugInstance>,

    /// Vulkan library loader
    entry: ash::Entry,
}

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("app_name", &self.app_name)
            .finish()
    }
}

impl Drop for DebugInstance {
    fn drop(&mut self) {
        unsafe {
            self._debug_utils.destroy_debug_utils_messenger(
                self._debug_messenger,
                self.allocation_callbacks.as_deref(),
            );
        };
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        tracing::debug!("Dropping instance");

        unsafe {
            self.raw
                .destroy_instance(self.allocation_callbacks.as_deref());
        };
    }
}

impl infrastructure::ResourceDeleter<Surface> for Instance {
    fn delete(&mut self, resource: &mut Surface) {
        unsafe {
            resource
                .surface_loader
                .destroy_surface(resource.raw, self.allocation_callbacks.as_deref())
        }
    }
}

pub struct DebugCallBackData {}

impl Instance {}

#[derive(Debug)]
pub struct InstanceCreateInfo {
    // TODO: change to &str?
    pub application_name: String,
    pub enabled_extensions: Vec<ExtensionName>,
    pub flags: ash::vk::InstanceCreateFlags,
    pub enabled_layers: Vec<ExtensionName>,
}

// TODO: create multiple default impls for different use cases
#[allow(dead_code, unreachable_code, unused_variables)]
impl InstanceCreateInfo {
    fn empty() -> Self {
        let flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            ash::vk::InstanceCreateFlags::default()
        };
        Self {
            application_name: "Example".to_string(),
            enabled_extensions: vec![],
            enabled_layers: vec![],
            flags,
        }
    }
    fn with_app_name(app_name: String) -> Self {
        Self {
            application_name: app_name,
            ..Self::empty()
        }
    }

    fn graphics_usage(app_name: String) -> Self {
        todo!();
        Self {
            application_name: app_name,
            ..Self::empty()
        }
    }
    fn compute_usage(app_name: String) -> Self {
        todo!();
        Self {
            application_name: app_name,
            ..Self::empty()
        }
    }
    fn mixed_usage(app_name: String) -> Self {
        todo!();
        Self {
            application_name: app_name,
            ..Self::empty()
        }
    }
}

impl Default for InstanceCreateInfo {
    fn default() -> Self {
        Self::empty()
    }
}

impl Instance {
    pub fn new_linked(info: InstanceCreateInfo) -> Result<Arc<Self>, error::InitError> {
        tracing::debug!("Creating VulkanLibrary");
        let entry = ash::Entry::linked();
        Instance::create_instance(entry, info)
    }
    pub fn new_dynamic(info: InstanceCreateInfo) -> Result<Arc<Self>, error::InitError> {
        tracing::debug!("Creating VulkanLibrary");
        let entry = unsafe { ash::Entry::load() }?;
        Instance::create_instance(entry, info)
    }

    pub fn create_surface<T>(
        self: &Arc<Instance>,
        handle: &T,
    ) -> Result<std::rc::Rc<Surface>, error::InitError>
    where
        T: raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle,
    {
        tracing::debug!("Creating surface");
        let display_handle = raw_window_handle::HasDisplayHandle::display_handle(&handle)?;
        let window_handle = raw_window_handle::HasWindowHandle::window_handle(&handle)?;
        let surface = unsafe {
            // platform independent
            ash_window::create_surface(
                &self.entry,
                &self.raw,
                *display_handle.as_ref(),
                *window_handle.as_ref(),
                self.allocation_callbacks.as_deref(),
            )
            .map_err(Into::<error::VkError>::into)?
        };
        Ok(std::rc::Rc::new(Surface {
            raw: surface,
            surface_loader: ash::khr::surface::Instance::new(&self.entry, &self.raw),
        }))
    }

    pub fn get_surface_required_extensions(
        handle: impl raw_window_handle::HasDisplayHandle,
    ) -> Result<Vec<ExtensionName>, error::InitError> {
        tracing::debug!("Getting surface required extensions");
        let display_handle = raw_window_handle::HasDisplayHandle::display_handle(&handle)?;
        let surface_extensions =
            ash_window::enumerate_required_extensions(*display_handle.as_ref())
                .map_err(Into::<error::VkError>::into)?;

        Ok(surface_extensions
            .iter()
            .map(|ext| ExtensionName::from_cstr(unsafe { CStr::from_ptr(*ext) }))
            .collect::<Vec<_>>())
    }

    // TODO: change this to validate the create info, rather than set a baseline
    // fn collect_instance_info(
    //     entry: &ash::Entry,
    //     required_extensions: Vec<ExtensionName>,
    // ) -> Result<InstanceCreateInfo, error::Error> {
    //     tracing::debug!("Collecting Vulkan instance info for ");
    //     let required_extensions = {
    //         let mut res = required_extensions;
    //         if cfg!(any(target_os = "macos", target_os = "ios")) {
    //             res.push(ash::vk::KHR_PORTABILITY_ENUMERATION_NAME.into());
    //             // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
    //             res.push(ash::vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.into());
    //         }
    //         res
    //     };
    //     tracing::trace!("Required extensions {required_extensions:?}");
    //     let optional_extensions = vec![ash::ext::debug_utils::NAME]
    //         .into_iter()
    //         .map(|x| x.into())
    //         .collect();
    //     let optional_extensions =
    //         Instance::check_if_extensions_are_supported(&required_extensions, optional_extensions)?;
    //     let enabled_extensions = required_extensions
    //         .into_iter()
    //         .chain(optional_extensions)
    //         .collect::<Vec<_>>();
    //     tracing::debug!("Optional extensions {:?}", enabled_extensions);
    //     let layers = if cfg!(not(build_type = "dist")) {
    //         vec![Layer::VALIDATIONLAYER]
    //     } else {
    //         vec![]
    //     };
    //     let validation_layers = self.filter_available_validation_layers(layers)?;
    //     tracing::debug!("Validation layers {:?}", validation_layers);
    //     let info = InstanceCreateInfo {
    //         enabled_layers: validation_layers,
    //         enabled_extensions,
    //         ..Default::default()
    //     };
    //     Ok(info)
    // }
    pub(crate) fn get_physical_device_properties(
        &self,
        device: &ash::vk::PhysicalDevice,
    ) -> ash::vk::PhysicalDeviceProperties {
        unsafe { self.raw.get_physical_device_properties(*device) }
    }
    pub(crate) fn get_physical_device_features(
        &self,
        device: &ash::vk::PhysicalDevice,
    ) -> ash::vk::PhysicalDeviceFeatures {
        unsafe { self.raw.get_physical_device_features(*device) }
    }
    pub(crate) fn get_physical_device_queue_family_properties(
        &self,
        physical_device: &ash::vk::PhysicalDevice,
    ) -> Vec<ash::vk::QueueFamilyProperties> {
        unsafe {
            self.raw
                .get_physical_device_queue_family_properties(*physical_device)
        }
    }

    pub(crate) fn enumerate_device_extension_properties(
        &self,
        device: &ash::vk::PhysicalDevice,
    ) -> Result<Vec<ExtensionProperties>, error::InitError> {
        let res = unsafe { self.raw.enumerate_device_extension_properties(*device) }?
            .into_iter()
            .map(Into::<ExtensionProperties>::into)
            .collect::<Vec<_>>();
        Ok(res)
    }

    fn create_allocation_call_back(
        _entry: &ash::Entry,
    ) -> Option<ash::vk::AllocationCallbacks<'static>> {
        None
    }

    fn create_instance(
        entry: ash::Entry,
        info: InstanceCreateInfo,
    ) -> Result<Arc<Instance>, error::InitError> {
        let _s = tracing::debug_span!("Instance creation");
        tracing::trace!("Instance creation info {:?}", info);
        let app_name = raw::string_to_cstring_remove_nuls(&info.application_name);
        let app_info = ash::vk::ApplicationInfo::default()
            .application_name(&app_name)
            .api_version(super::constants::VULKAN_API_VERSION)
            .engine_version(super::constants::ENGINE_VERSION)
            .engine_name(super::constants::ENGINE_NAME);

        let enabled_extensions = info
            .enabled_extensions
            .iter()
            .map(|ext| ext.to_str().as_ptr())
            .collect::<Vec<_>>();

        let enabled_layer_names = info
            .enabled_layers
            .iter()
            .map(|ext| ext.to_str().as_ptr())
            .collect::<Vec<_>>();

        let allocation_call_back =
            Instance::create_allocation_call_back(&entry).map(std::rc::Rc::new);

        let mut debug_messenger_data = Box::new(DebugCallBackData {});
        let create_info = ash::vk::InstanceCreateInfo::default()
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layer_names)
            .flags(info.flags)
            .application_info(&app_info);

        // TODO: check if this is correct?
        let instance = unsafe {
            entry
                .create_instance(&create_info, allocation_call_back.as_deref())
                .map_err(Into::<error::VkError>::into)?
        };

        #[cfg(not(build_type = "dist"))]
        let debug_messenger_instance = {
            let debug_creation_info = Self::messenger_create_info(&mut debug_messenger_data);
            let debug_utils_loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
            let debug_call_back = unsafe {
                debug_utils_loader
                    .create_debug_utils_messenger(
                        &debug_creation_info,
                        allocation_call_back.as_deref(),
                    )
                    .unwrap()
            };
            DebugInstance {
                _debug_utils: debug_utils_loader,
                _debug_messenger: debug_call_back,
                _callback_data: debug_messenger_data,
                allocation_callbacks: allocation_call_back.clone(),
            }
        };

        Ok(Arc::new(Instance {
            raw: instance,
            allocation_callbacks: allocation_call_back,
            app_name,
            #[cfg(not(build_type = "dist"))]
            debug_messenger_instance: Some(debug_messenger_instance),
            entry,
        }))
    }
    /// [`layer_name`] The layer to retrieve extensions from
    pub fn enumerate_instance_extension_properties(
        self: &Arc<Self>,
        layer_name: Option<&ExtensionName>,
    ) -> Result<Vec<ExtensionProperties>, error::InitError> {
        let res = unsafe {
            self.entry
                .enumerate_instance_extension_properties(layer_name.map(|ext| ext.deref()))
        }
        .map(|v| {
            v.into_iter()
                .map(Into::<ExtensionProperties>::into)
                .collect::<Vec<_>>()
        })?;
        Ok(res)
    }

    /// Ensures that all required and optional extensions are supported by the vulkan implementation. Throws an error if any of the required extensions are not supported.
    /// Returns a list of supported optional extensions.
    /// # Error
    /// Returns which required extensions are not supported.
    pub(crate) fn check_if_extensions_are_supported(
        self: &Arc<Self>,
        required_extensions: &[ExtensionName],
        optional_extensions: Vec<ExtensionName>,
    ) -> Result<Vec<ExtensionName>, error::InitError> {
        let available_extensions = self
            // TODO: what is this none?
            .enumerate_instance_extension_properties(None)?;
        let all_required_met = required_extensions
            .iter()
            .all(|ext| available_extensions.iter().any(|av| &av.name == ext));

        if all_required_met {
            Ok(optional_extensions
                .into_iter()
                .filter(|ext| available_extensions.iter().any(|av| &av.name == ext))
                .collect::<Vec<_>>())
        } else {
            // Collect all the missing extensions
            Err(error::InitError::RequiredExtensionsMissing(
                required_extensions
                    .iter()
                    .filter(|&ext| !available_extensions.iter().any(|av| &av.name == ext))
                    .cloned()
                    .collect::<Vec<_>>(),
            ))
        }
    }

    fn enumerate_instance_layer_properties(&self) -> Result<Vec<Layer>, error::InitError> {
        let r = unsafe { self.entry.enumerate_instance_layer_properties() }
            .map_err(Into::<error::VkError>::into)
            .map(|v| v.into_iter().map(Into::<Layer>::into).collect::<Vec<_>>())?;
        Ok(r)
    }
    pub(crate) fn filter_available_validation_layers(
        &self,
        validation_layers: Vec<ExtensionName>,
    ) -> Result<Vec<ExtensionName>, error::InitError> {
        let layers = self.enumerate_instance_layer_properties()?;
        let result = validation_layers
            .into_iter()
            .filter(|layer| {
                let x = layers.iter().any(|layer2| layer2.name == *layer);
                if !x {
                    tracing::warn!("Validation layer {:?} is not available", layer);
                }
                x
            })
            .collect();
        Ok(result)
    }
    fn message_severity() -> ash::vk::DebugUtilsMessageSeverityFlagsEXT {
        let val = {
            #[cfg(build_type = "debug")]
            {
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            }
            #[cfg(not(build_type = "debug"))]
            {
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            }
        };
        tracing::info!("Vulkan message callback severity set to: {:?}", val);
        val
    }
    fn message_type() -> ash::vk::DebugUtilsMessageTypeFlagsEXT {
        let val = {
            #[cfg(build_type = "dist")]
            {
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            }
            #[cfg(build_type = "debug")]
            {
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            }
            #[cfg(build_type = "release")]
            {
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            }
        };
        tracing::info!("Vulkan message callback types set to: {:?}", val);
        val
    }

    fn messenger_create_info(
        callback_data: &mut Box<DebugCallBackData>,
    ) -> ash::vk::DebugUtilsMessengerCreateInfoEXT {
        ash::vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(Instance::message_severity())
            .message_type(Instance::message_type())
            .pfn_user_callback(Some(vulkan_debug_callback))
            .user_data(
                (callback_data.as_mut()) as *mut DebugCallBackData as *mut std::os::raw::c_void,
            )
    }
}
/// A lambda function you can pass to vulkan.
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT,
    user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 {
    use std::ffi::CStr;
    let _debug_callback_data: &mut DebugCallBackData =
        unsafe { *(user_data as *mut &mut DebugCallBackData) };

    let callback_data = *p_callback_data;
    // let message_id_number = callback_data.message_id_number;

    let _message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    let _objects = if callback_data.p_objects.is_null() {
        None
    } else {
        let objects = std::slice::from_raw_parts(
            callback_data.p_objects,
            callback_data.object_count as usize,
        );
        Some(objects)
    };
    let msg = format!("{}", message);
    // let msg = format!("{}: [{}] ({:?})", message_id_name, message, objects);
    match message_severity {
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            tracing::error!(msg)
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            tracing::warn!(msg)
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            tracing::info!(msg)
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            tracing::debug!(msg)
        }
        _ => {}
    }

    ash::vk::FALSE
}
