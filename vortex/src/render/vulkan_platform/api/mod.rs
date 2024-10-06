#![allow(dead_code)]
pub mod constants;
pub mod error;
pub mod memory;
pub mod raw;

use raw::from_cstr_to_array;
use std::{ffi::CStr, ops::Deref};

/// Statically linked vulkan library at compile time.
pub struct VulkanLibrary {
    pub entry: ash::Entry,
}

pub struct DebugCallBackData {}

pub struct Instance {
    raw: ash::Instance,
    pub allocation_callbacks: Option<ash::vk::AllocationCallbacks<'static>>,
    pub app_name: std::ffi::CString,
    _debug_utils: ash::ext::debug_utils::Instance,
    _debug_messenger: ash::vk::DebugUtilsMessengerEXT,
    pub callback_data: Box<DebugCallBackData>,
    // _debug_utils: ash::extensions::ext::DebugUtils,
    // _debug_messenger: ash::vk::DebugUtilsMessengerEXT,
}

#[derive(derive_more::Deref)]
pub struct Surface {
    #[deref]
    pub surface: ash::vk::SurfaceKHR,
}
#[derive(Debug, smart_default::SmartDefault)]
pub struct InstanceCreateInfo<'a> {
    #[default = "Example"]
    pub application_name: &'a str,
    #[default(vec![])]
    pub enabled_extensions: Vec<ExtensionProperties>,
    #[default({
        if cfg!(any(target_os = "macos", target_os = "ios")) { ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR } else {
            ash::vk::InstanceCreateFlags::default()
        }
    })]
    pub flags: ash::vk::InstanceCreateFlags,
    #[default(vec![])]
    pub enabled_layers: Vec<Layer>,
}

#[derive(
    Debug,
    derive_more::From,
    derive_more::Into,
    derive_more::AsRef,
    derive_more::AsMut,
    PartialEq,
    Clone,
)]
pub struct ExtensionName([i8; ExtensionName::MAX_SIZE]);
impl ExtensionName {
    pub const MAX_SIZE: usize = ash::vk::MAX_EXTENSION_NAME_SIZE;
    pub const fn from_cstr(cstr: &CStr) -> Self {
        Self(from_cstr_to_array(cstr))
    }
}

impl std::ops::Deref for ExtensionName {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }
}
#[derive(
    Debug,
    derive_more::From,
    derive_more::Into,
    derive_more::AsRef,
    derive_more::AsMut,
    PartialEq,
    Clone,
)]
pub struct Description([i8; Description::MAX_SIZE]);
impl Description {
    pub const MAX_SIZE: usize = ash::vk::MAX_DESCRIPTION_SIZE;
    pub const fn from_cstr(cstr: &CStr) -> Self {
        Self(from_cstr_to_array(cstr))
    }
}

impl std::ops::Deref for Description {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: ExtensionName,
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: Description,
}
impl Layer {
    pub const VALIDATIONLAYER: ExtensionName =
        ExtensionName::from_cstr(c"VK_LAYER_KHRONOS_validation");
}

impl From<ash::vk::LayerProperties> for Layer {
    fn from(prop: ash::vk::LayerProperties) -> Self {
        Self {
            name: ExtensionName(prop.layer_name),
            spec_version: prop.spec_version,
            implementation_version: prop.implementation_version,
            description: Description(prop.description),
        }
    }
}

#[derive(Debug, derive_more::From, derive_more::Into, PartialEq)]
pub struct ExtensionProperties {
    pub name: ExtensionName,
    pub spec_version: u32,
}

impl From<ash::vk::ExtensionProperties> for ExtensionProperties {
    fn from(prop: ash::vk::ExtensionProperties) -> Self {
        Self {
            name: ExtensionName(prop.extension_name),
            spec_version: prop.spec_version,
        }
    }
}

// Due to the many linking chains, this type simply outputs the handle when printed using Debug.
#[derive(
    derive_more::From,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
)]
pub struct HandleWrapper<T: ash::vk::Handle>(T);

impl<T: ash::vk::Handle> std::fmt::Debug for HandleWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle: {:?}", <T as ash::vk::Handle>::TYPE)
    }
}

pub struct SwapChainSupport {
    pub capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub extent: ash::vk::Extent2D,
    pub format: ash::vk::SurfaceFormatKHR,
    pub present_mode: ash::vk::PresentModeKHR,
}

impl VulkanLibrary {
    pub fn new() -> Result<Self, error::Generic> {
        let entry = ash::Entry::linked();
        Ok(Self { entry })
    }

    pub fn create_surface(
        &self,
        instance: &Instance,
        window: &crate::window::Window,
    ) -> Result<Surface, error::Generic> {
        let display_handle = raw_window_handle::HasDisplayHandle::display_handle(&window)?;
        let window_handle = raw_window_handle::HasWindowHandle::window_handle(&window)?;
        let surface = unsafe {
            // platform independent
            ash_window::create_surface(
                &self.entry,
                &instance.raw,
                *display_handle.as_ref(),
                *window_handle.as_ref(),
                instance.allocation_callbacks.as_ref(),
            )
            .map_err(Into::<error::VkError>::into)?
        };
        Ok(Surface {
            surface,
            //surface_loader: ash::khr::Surface::new(&self.entry, &instance.inner),
        })
    }

    pub fn get_surface_required_extensions(
        window: impl raw_window_handle::HasDisplayHandle,
    ) -> Result<Vec<ExtensionName>, error::Generic> {
        let display_handle = raw_window_handle::HasDisplayHandle::display_handle(&window)?;
        let surface_extensions =
            ash_window::enumerate_required_extensions(*display_handle.as_ref())
                .map_err(Into::<error::VkError>::into)?;

        Ok(surface_extensions
            .iter()
            .map(|ext| ExtensionName::from_cstr(unsafe { CStr::from_ptr(*ext) }))
            .collect::<Vec<_>>())
    }

    fn create_allocation_call_back(&self) -> Option<ash::vk::AllocationCallbacks<'static>> {
        None
    }

    pub fn create_instance(&self, info: InstanceCreateInfo) -> Result<Instance, error::Generic> {
        let app_name = raw::string_to_cstring_remove_nuls(info.application_name);
        let app_info = ash::vk::ApplicationInfo::default()
            .application_name(&app_name)
            .api_version(constants::VULKAN_API_VERSION)
            .engine_version(constants::ENGINE_VERSION)
            .engine_name(constants::ENGINE_NAME);

        let enabled_extensions = info
            .enabled_extensions
            .iter()
            .map(|ext| ext.name.0.as_ptr())
            .collect::<Vec<_>>();

        let enabled_layer_names = info
            .enabled_layers
            .iter()
            .map(|ext| ext.name.0.as_ptr())
            .collect::<Vec<_>>();

        let allocation_call_back = self.create_allocation_call_back();

        let mut debug_messenger_data = Box::new(DebugCallBackData {});
        let mut debug_info_creation_info = Self::messenger_create_info(&mut debug_messenger_data);
        let mut create_info = ash::vk::InstanceCreateInfo::default()
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layer_names)
            .flags(info.flags)
            .application_info(&app_info);

        // TODO: check if this is correct?
        create_info = create_info.push_next(&mut debug_info_creation_info);
        let instance = unsafe {
            self.entry
                .create_instance(&create_info, allocation_call_back.as_ref())
                .map_err(Into::<error::VkError>::into)?
        };

        let debug_creation_info2 = Self::messenger_create_info(&mut debug_messenger_data);
        let debug_utils_loader = ash::ext::debug_utils::Instance::new(&self.entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_creation_info2, allocation_call_back.as_ref())
                .unwrap()
        };

        Ok(Instance {
            raw: instance,
            allocation_callbacks: allocation_call_back,
            app_name,
            _debug_utils: debug_utils_loader,
            _debug_messenger: debug_call_back,
            callback_data: debug_messenger_data,
        })
    }
    pub fn enumerate_instance_extension_properties(
        &self,
        layer_name: Option<&ExtensionName>,
    ) -> Result<Vec<ExtensionProperties>, error::Generic> {
        let res = unsafe {
            self.entry
                .enumerate_instance_extension_properties(layer_name.map(|ext| ext.deref()))
        }
        .map_err(Into::<error::VkError>::into)
        .map(|v| {
            v.into_iter()
                .map(Into::<ExtensionProperties>::into)
                .collect::<Vec<_>>()
        })?;
        Ok(res)
    }

    /// Ensures that all required and optional extensions are supported by the vulkan implementation. Throws an error if any of the required extensions are not supported.
    /// Optionally returns a list of supported optional extensions.
    pub(crate) fn check_if_extensions_are_supported(
        &self,
        required_extensions: &[ExtensionName],
        optional_extensions: Vec<ExtensionName>,
    ) -> Result<Vec<ExtensionName>, error::Generic> {
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
            Err(error::Generic::RequiredExtensionsMissing(
                required_extensions
                    .iter()
                    .filter(|&ext| !available_extensions.iter().any(|av| &av.name == ext))
                    .cloned()
                    .collect::<Vec<_>>(),
            ))
        }
    }

    fn enumerate_instance_layer_properties(&self) -> Result<Vec<Layer>, error::Generic> {
        let r = unsafe { self.entry.enumerate_instance_layer_properties() }
            .map_err(Into::<error::VkError>::into)
            .map(|v| v.into_iter().map(Into::<Layer>::into).collect::<Vec<_>>())?;
        Ok(r)
    }
    pub(crate) fn filter_available_validation_layers(
        &self,
        validation_layers: Vec<Layer>,
    ) -> Result<Vec<Layer>, error::Generic> {
        let layers = self.enumerate_instance_layer_properties()?;
        let result = validation_layers
            .into_iter()
            .filter(|layer| {
                let x = layers.iter().any(|layer2| layer2.name == layer.name);

                if !x {
                    crate::log::warn!("Validation layer {:?} is not available", layer);
                }
                x
            })
            .collect();
        Ok(result)
    }
    fn message_severity() -> ash::vk::DebugUtilsMessageSeverityFlagsEXT {
        // TODO:
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
    }
    fn message_type() -> ash::vk::DebugUtilsMessageTypeFlagsEXT {
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
    }

    fn messenger_create_info(
        callback_data: &mut Box<DebugCallBackData>,
    ) -> ash::vk::DebugUtilsMessengerCreateInfoEXT {
        ash::vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(VulkanLibrary::message_severity())
            .message_type(VulkanLibrary::message_type())
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
            crate::log::error!(msg)
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            crate::log::warn!(msg)
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            crate::log::info!(msg)
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            crate::log::debug!(msg)
        }
        _ => {}
    }

    ash::vk::FALSE
}
