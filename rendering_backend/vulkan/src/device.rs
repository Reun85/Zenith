use crate::instance::Instance;

use super::error;
use super::surface;
use super::types::{ExtensionName, Layer};
use std::sync::Arc;

pub struct QueueIndex(u32);

pub struct Queue {
    inner: ash::vk::Queue,
    family_index: QueueIndex,
    capabilities: ash::vk::QueueFlags,
}

pub struct Device {
    raw: ash::Device,
    physical_device: ash::vk::PhysicalDevice,
    queues: Vec<Queue>,
    instance: Arc<Instance>, // NOTE: This could be circumvented by using by creating a god instance, that holds reference
                             // to everything related to vulkan, and manually drops everything.
}
pub struct DeviceCreationInfo {
    instance: Arc<Instance>,
    physical_device_creation_info: PhysicalDeviceCreationInfo,
    required_extensions: Vec<ExtensionName>,
    required_queues: Vec<ash::vk::QueueFlags>,
    /// build process returns which it was able to get.
    optional_extensions: Vec<ExtensionName>,
    optional_queues: Vec<ExtensionName>,
}

pub struct PhysicalDeviceCreationInfo {}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw
                .destroy_device(self.instance.allocation_callbacks.as_deref())
        };
    }
}
impl Device {
    pub fn new(
        instance: &Arc<Instance>,
        surface: &surface::Surface,
        layers: Vec<Layer>,
        extensions: Vec<ExtensionName>,
    ) -> Result<Arc<Self>, error::Error> {
        let physical_device = Self::pick_physical_device(instance, surface)?;
        let (graphic_family_index, present_family_index) =
            Self::find_queue_families(instance, &physical_device, surface)?;
        let device = Self::create_logical_device(
            instance,
            &physical_device,
            graphic_family_index,
            present_family_index,
            layers,
            extensions,
        )?;
        let graphics_queue = unsafe { device.get_device_queue(graphic_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };
        Ok(Arc::new(Self {
            raw: device,
            instance: instance.clone(),
            physical_device,
            graphics_queue,
            present_queue,
            graphic_family_index,
            present_family_index,
        }))
    }
    fn pick_physical_device(
        instance: &Arc<Instance>,
        surface: &surface::Surface,
    ) -> Result<ash::vk::PhysicalDevice, error::Error> {
        let physical_devices = unsafe { instance.raw.enumerate_physical_devices() }
            .map_err(|_| error::Error::EnumeratePhysicalDevicesFailed)?;
        let physical_device = physical_devices
            .iter()
            .find(|physical_device| Self::is_device_suitable(instance, **physical_device, surface))
            .copied()
            .ok_or(error::Error::SuitablePhysicalDeviceNotFound)?;
        Ok(physical_device)
    }
    fn is_device_suitable(
        instance: &Arc<Instance>,
        physical_device: ash::vk::PhysicalDevice,
        surface: &surface::Surface,
    ) -> bool {
        let properties = unsafe { instance.raw.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.raw.get_physical_device_features(physical_device) };
        let indices = Self::find_queue_families(instance, &physical_device, surface);
        let extensions = unsafe {
            instance
                .raw
                .enumerate_device_extension_properties(physical_device)
        }
        .map(|extensions| {
            extensions
                .into_iter()
                .map(|extension| extension.extension_name)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
        let supported_extensions = instance.supported_extensions();
        let supported_layers = instance.supported_layers();
        let is_queue_families_supported = indices.is_ok();
        //let is_extensions;
        true
    }
}
