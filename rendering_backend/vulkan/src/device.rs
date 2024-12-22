//! All the necessary Rust binding for Vulkan attributes
use crate::instance::Instance;
use crate::types::ExtensionProperties;
use crate::types::Layer;
use infrastructure::Promise;

use super::error;
use super::types::ExtensionName;
use std::rc::Rc;
use std::sync::Arc;

pub use ash::vk::QueueFlags;

#[derive(Debug, Clone, Default)]
pub struct QueueDescription {
    pub flags: ash::vk::QueueFlags,
    pub priorities: Vec<f32>,
    pub supports: Option<SwapChainPromise>,
}

impl QueueDescription {
    /// A queue description with a given number of returned queues.
    /// These queues will have priority 0
    pub fn from_count(count: u16) -> QueueDescription {
        QueueDescription {
            priorities: (0..count).map(|_| 0.0).collect(),
            ..Default::default()
        }
    }
    /// A queue description with priority list.
    pub fn from_priorities(vec: Vec<f32>) -> QueueDescription {
        QueueDescription {
            priorities: vec,
            ..Default::default()
        }
    }
}

/// GPU Queue
#[derive(Debug)]
pub struct Queue {
    inner: ash::vk::Queue,
    capabilities: ash::vk::QueueFlags,
}

type QueuePromise = Rc<Promise<Rc<Queue>, QueueDescription>>;
impl Queue {
    pub fn new_promise(desc: QueueDescription) -> QueuePromise {
        Promise::new_rc(desc)
    }
}

#[derive(Debug, Clone)]
pub struct SwapChainDescription {
    pub surface: Rc<super::Surface>,
}

#[derive(Debug)]
pub struct SwapChain {
    inner: ash::vk::SwapchainKHR,
    surface: Rc<super::Surface>,
    capabilities: ash::vk::QueueFlags,
}

type SwapChainPromise = Rc<Promise<Rc<SwapChain>, SwapChainDescription>>;
impl SwapChain {
    pub fn new_promise(desc: SwapChainDescription) -> SwapChainPromise {
        Promise::new_rc(desc)
    }
}

pub struct Device {
    raw: ash::Device,
    physical_device: ash::vk::PhysicalDevice,
    queues: Vec<Rc<Queue>>,
    instance: Arc<Instance>,
}
unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw
                .destroy_device(self.instance.allocation_callbacks.as_deref());
        }
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("raw", &self.raw.handle())
            .field("physical_device", &self.physical_device)
            .field("queues", &self.queues)
            .field("instance", &self.instance)
            .finish()
    }
}

/**

Order the devices.
Returning `None` means that the device is not suitable.
Otherwise higher the number, higher the priority.
Errors are not propagated back, if it does not match the predicate make sure to log it as an
error!
*/
pub struct DeviceOrdering(pub fn(&mut PhysicalDeviceProperties) -> Option<u32>);

impl Default for DeviceOrdering {
    /// Must support all extensions.
    fn default() -> Self {
        fn default_device_ordering(desc: &mut PhysicalDeviceProperties) -> Option<u32> {
            let mut fail: bool = false;
            let mut score = 0;

            fail |= desc.extensions.iter().any(|x| !x.is_supported());
            fail |= desc.queues.descriptions.iter().any(|x| !x.is_supported());
            // if desc.features.geometry_shader == ash::vk::Bool32::from(false) {
            //     fail = true;
            // }
            // if desc.features.sampler_anisotropy == ash::vk::Bool32::from(false) {
            //     fail = true;
            // }

            score += match desc.properties.device_type {
                ash::vk::PhysicalDeviceType::DISCRETE_GPU => 4,
                ash::vk::PhysicalDeviceType::INTEGRATED_GPU => 3,
                ash::vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                ash::vk::PhysicalDeviceType::CPU => 1,
                _ => 0,
            };

            match fail {
                true => None,
                false => Some(score),
            }
        }
        Self(default_device_ordering)
    }
}

pub struct DeviceCreationInfo {
    pub physical_device_creation_info: PhysicalDeviceCreationInfo,
    pub extensions: Vec<ExtensionName>,
    pub layers: Vec<Layer>,
    /// After using `Device::new` these promises will be filled out.
    pub queues: Vec<QueuePromise>,
    pub swapchain: Vec<SwapChainPromise>,
    /// If you can work with missing extensions, overdefine this variable
    /// set it to change variables in your domain and handle it there.
    pub order: DeviceOrdering,
}

pub struct SwapChainSupport {
    pub capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub extent: ash::vk::Extent2D,
    pub format: ash::vk::SurfaceFormatKHR,
    pub present_mode: ash::vk::PresentModeKHR,
}

#[derive(Default)]
pub struct PhysicalDeviceCreationInfo {}

enum Support<Value, ID> {
    Supported((Value, ID)),
    Unsupported(ID),
}
impl<Value, ID> Support<Value, ID> {
    fn is_supported(&self) -> bool {
        match self {
            Self::Supported(_) => true,
            Self::Unsupported(_) => false,
        }
    }
}

impl<Value: std::fmt::Debug, ID: std::fmt::Debug> std::fmt::Debug for Support<Value, ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Supported((value, id)) => {
                f.debug_tuple("Supported").field(value).field(id).finish()
            }
            Self::Unsupported(id) => f.debug_tuple("Unsupported").field(id).finish(),
        }
    }
}

type Indices = u16;
struct QueueSupportData {
    queue_datas: Vec<ash::vk::QueueFamilyProperties>,
    /// The indices are for the previous array.
    descriptions: Vec<Support<Vec<Indices>, QueuePromise>>,
}
pub struct PhysicalDeviceProperties {
    /// List of requested items.
    extensions: Vec<Support<ExtensionProperties, ExtensionName>>,
    queues: QueueSupportData,
    properties: ash::vk::PhysicalDeviceProperties,
    features: ash::vk::PhysicalDeviceFeatures,
}

struct PickPhysicalDevice {
    device: ash::vk::PhysicalDevice,
    extensions: Vec<ExtensionName>,
    queues: Vec<Queue>,
}

mod physical {

    use super::*;
    pub fn get_init_details(
        instance: &Arc<Instance>,
        physical_device: &ash::vk::PhysicalDevice,
        extensions: &[ExtensionName],
        queues: &[QueuePromise],
    ) -> Result<PhysicalDeviceProperties, error::InitError> {
        let properties = instance.get_physical_device_properties(physical_device);
        let features = instance.get_physical_device_features(physical_device);
        let queue_family_properties =
            instance.get_physical_device_queue_family_properties(physical_device);
        let supported_extensions =
            instance.enumerate_device_extension_properties(physical_device)?;

        let extensions = extensions
            .iter()
            .map(|extension| {
                supported_extensions
                    .iter()
                    .find(|x| x.name == *extension)
                    .map(|x| Support::Supported((x.clone(), extension.clone())))
                    .unwrap_or_else(|| Support::Unsupported(extension.clone()))
            })
            .collect();

        let queue_data = {
            fn queue_matches_description(
                x: &ash::vk::QueueFamilyProperties,
                desc: &QueueDescription,
            ) -> bool {
                x.queue_count > 0 && x.queue_flags.contains(desc.flags)
            }
            let queue_families = queue_family_properties
                .into_iter()
                .filter(|x| {
                    queues
                        .iter()
                        .any(|desc| queue_matches_description(x, &desc.description))
                })
                .collect::<Vec<_>>();

            let indices = queues
                .iter()
                .map(|desc| {
                    let matching_families: Vec<_> = queue_families
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| queue_matches_description(x, &desc.description))
                        .map(|(ind, _)| ind as u16)
                        .collect();
                    match matching_families.is_empty() {
                        true => Support::Unsupported((*desc).clone()),
                        false => Support::Supported((matching_families, (*desc).clone())),
                    }
                })
                .collect();
            QueueSupportData {
                queue_datas: queue_families,
                descriptions: indices,
            }
        };
        let res = PhysicalDeviceProperties {
            extensions,
            queues: queue_data,
            properties,
            features,
        };
        Ok(res)
    }

    fn query_surface_support(
        surface: &crate::instance::Surface,
        physical_device: ash::vk::PhysicalDevice,
        window_inner_size: (u32, u32),
    ) -> Option<SwapChainSupport> {
        let formats = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface.raw)
        }
        .unwrap();
        let present_modes = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface.raw)
        }
        .unwrap();
        if !(!formats.is_empty() && !present_modes.is_empty()) {
            return None;
        }
        let best_format = formats
            .into_iter()
            .max_by_key(|x| {
                let mut score = match x.format {
                    ash::vk::Format::B8G8R8A8_SRGB => 1,
                    _ => 0,
                };
                score += match x.color_space {
                    ash::vk::ColorSpaceKHR::SRGB_NONLINEAR => 2,
                    _ => 0,
                };
                score
            })
            .unwrap();
        let best_present_mode = present_modes
            .into_iter()
            .max_by_key(|x| {
                match *x {
                    // isn't this guaranteed to be available?
                    ash::vk::PresentModeKHR::FIFO => 3,
                    ash::vk::PresentModeKHR::MAILBOX => 2,
                    ash::vk::PresentModeKHR::IMMEDIATE => 1,
                    _ => 0,
                }
            })
            .unwrap();
        let present_capabilities = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface.raw)
        }
        .unwrap();
        let mut extent: ash::vk::Extent2D = if present_capabilities.current_extent.width != u32::MAX
        {
            ash::vk::Extent2D {
                width: window_inner_size.0,
                height: window_inner_size.1,
            }
        } else {
            present_capabilities.current_extent
        };
        extent.width = extent.width.clamp(
            present_capabilities.min_image_extent.width,
            present_capabilities.max_image_extent.width,
        );
        extent.height = extent.height.clamp(
            present_capabilities.min_image_extent.height,
            present_capabilities.max_image_extent.height,
        );
        let swap_chain_support = SwapChainSupport {
            format: best_format,
            present_mode: best_present_mode,
            extent,
            capabilities: present_capabilities,
        };
        Some(swap_chain_support)
    }
}
impl Device {
    pub fn new(
        instance: &Arc<Instance>,
        info: DeviceCreationInfo,
    ) -> Result<Arc<Self>, error::InitError> {
        let physical_devices = unsafe { instance.raw.enumerate_physical_devices() }
            .map_err(|_| error::InitError::EnumeratePhysicalDevicesFailed)?;
        let physical_device = physical_devices
            .into_iter()
            .filter_map(|physical_device| {
                let res = physical::get_init_details(
                    instance,
                    &physical_device,
                    &info.extensions,
                    &info.queues,
                );

                match res {
                    Ok(properties) => Some((physical_device, properties)),
                    Err(x) => {
                        tracing::error!(
                            "Physical device properties could not be parsed: {:?}. Ignoring",
                            x
                        );
                        None
                    }
                }
            })
            .filter_map(|(physical_device, mut properties)| {
                (info.order.0)(&mut properties).map(|score| (physical_device, properties, score))
            })
            .max_by_key(|(_, _, score)| *score)
            .map(|(physical_device, properties, _)| (physical_device, properties));

        let (physical_device, properties) =
            physical_device.ok_or(error::InitError::SuitablePhysicalDeviceNotFound)?;

        let extensions = properties
            .extensions
            .into_iter()
            .filter_map(|supp| match supp {
                Support::Unsupported(_) => None,
                Support::Supported((val, _)) => Some(val),
            })
            .collect::<Vec<_>>();
        let extension_names = extensions
            .iter()
            .map(|prop| prop.name.to_str().as_ptr())
            .collect::<Vec<_>>();
        let enabled_features = properties.features;

        let solved_queues = solve_queues(properties.queues)?;
        // TODO: This is not correct for multiple queues
        let queue_create_info = solved_queues
            .solved_queues
            .iter()
            .map(|x| {
                to_create_info(
                    solved_queues.queue_family_buffer.get(x.0 as usize).unwrap(),
                    &x.1.description,
                )
            })
            .collect::<Vec<_>>();

        let device_create_info = ash::vk::DeviceCreateInfo::default()
            .enabled_extension_names(&extension_names)
            .enabled_features(&enabled_features)
            .queue_create_infos(&queue_create_info);
        let device = unsafe {
            instance.raw.create_device(
                physical_device,
                &device_create_info,
                instance.allocation_callbacks.as_deref(),
            )?
        };

        let queues = solved_queues
            .solved_queues
            .iter()
            .map(|(family, desc)| {
                let queue = unsafe { device.get_device_queue(*family as u32, 0) };
                let q = Rc::new(Queue {
                    inner: queue,
                    capabilities: desc.description.flags,
                });
                *desc.result.borrow_mut() = Some(Rc::clone(&q));
                q
            })
            .collect::<Vec<_>>();
        let device = Self {
            raw: device,
            instance: instance.clone(),
            physical_device,
            queues,
        };
        Ok(Arc::new(device))
    }

    pub fn create_swapchain(
        &self,
        instance: &Arc<Instance>,
        surface: &crate::instance::Surface,
        details: SwapChainSupport,
    ) -> Result<SwapChain, error::InitError> {
        let image_count = (details.capabilities.min_image_count + 1).max(2);
        let max_image_count = if details.capabilities.max_image_count == 0 {
            u32::MAX
        } else {
            details.capabilities.max_image_count
        };
        let image_count = image_count.min(max_image_count);
        let (image_sharing_mode, queue_family_indices) =
        // TODO:This is bad, this means that only a single queue may access the swapchain at all.
            // if device.graphic_family_index != device.present_family_index {
            //     (
            //         ash::vk::SharingMode::CONCURRENT,
            //         vec![device.graphic_family_index, device.present_family_index],
            //     )
            // } else {
                (ash::vk::SharingMode::EXCLUSIVE, vec![])
            // }
        ;
        let create_info = ash::vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw)
            .min_image_count(image_count)
            .image_format(details.format.format)
            .image_color_space(details.format.color_space)
            .image_extent(details.extent)
            .image_array_layers(1)
            .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(details.capabilities.current_transform)
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(details.present_mode)
            .clipped(true);

        let swapchain_loader =
            unsafe { ash::khr::swapchain::Device::new(&self.instance.raw, &self.raw) };
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, self.instance.allocation_callbacks.as_deref())
                .map_err(|x| error::InitError::SwapchainCreationFailed(x.into()))
        }?;

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

        let image_views = swapchain_images
            .iter()
            .map(|x| self.create_image_view(x, details.format.format).unwrap())
            // .map(|image| {
            //     let create_info = ImageViewCreateInfo::builder()
            //         .image(*image)
            //         .view_type(ash::vk::ImageViewType::TYPE_2D)
            //         .format(details.format.format)
            //         .components(ash::vk::ComponentMapping {
            //             r: ash::vk::ComponentSwizzle::IDENTITY,
            //             g: ash::vk::ComponentSwizzle::IDENTITY,
            //             b: ash::vk::ComponentSwizzle::IDENTITY,
            //             a: ash::vk::ComponentSwizzle::IDENTITY,
            //         })
            //         .subresource_range(ash::vk::ImageSubresourceRange {
            //             aspect_mask: ash::vk::ImageAspectFlags::COLOR,
            //             base_mip_level: 0,
            //             level_count: 1,
            //             base_array_layer: 0,
            //             layer_count: 1,
            //         })
            //         .build();
            //     unsafe { device.device.create_image_view(&create_info, None) }.unwrap()
            // })
            .collect::<Vec<_>>();

        Ok(SwapChain {
            swapchain,
            swapchain_loader,
            _images: swapchain_images,
            image_views,
            extent: details.extent,
            surface_format: details.format,
        })
    }
}

fn to_create_info<'q>(
    family: &ash::vk::QueueFamilyProperties,
    desc: &'q QueueDescription,
) -> ash::vk::DeviceQueueCreateInfo<'q> {
    ash::vk::DeviceQueueCreateInfo::default()
        .queue_family_index(family.queue_flags.as_raw())
        .queue_priorities(&desc.priorities)
}

struct SolvedQueueLayout {
    queue_family_buffer: Vec<ash::vk::QueueFamilyProperties>,
    solved_queues: Vec<(u16, QueuePromise)>,
}
fn solve_queues(inp: QueueSupportData) -> Result<SolvedQueueLayout, error::InitError> {
    let QueueSupportData {
        queue_datas: mut queues,
        descriptions: indices,
    } = inp;

    if indices.is_empty() {
        return Ok(SolvedQueueLayout {
            queue_family_buffer: vec![],
            solved_queues: vec![],
        });
    }

    enum State {
        Unsolved((Vec<u16>, QueuePromise)),
        Solved((u16, QueuePromise)),
    }

    let mut res: Vec<State> = indices
        .into_iter()
        .filter_map(|support| match support {
            Support::Supported((indices, desc)) => Some(State::Unsolved((indices, desc))),
            Support::Unsupported(desc) => {
                tracing::error!(
                    "Queue description {:?} is not supported but got through filtering. Ignoring.",
                    desc
                );
                None
            }
        })
        .collect();

    let mut changed = true;

    while changed && res.iter().any(|x| matches!(x, State::Unsolved(_))) {
        changed = false;

        res = res
            .into_iter()
            .map(|entry| {
                match entry {
                    State::Unsolved((mut possible_indices, desc)) => {
                        // Find the first available queue family with remaining capacity
                        let mut selected_family = None;

                        for index in possible_indices.iter() {
                            if let Some(family) = queues.get_mut(*index as usize) {
                                if family.queue_count > 0 {
                                    break;
                                }
                                selected_family = Some(*index);
                            }
                        }

                        match selected_family {
                            Some(index) => {
                                queues.get_mut(index as usize).unwrap().queue_count -= 1;
                                possible_indices.retain(|&i| i != index); // Avoid reusing the same family
                                changed = true;
                                State::Solved((index, desc))
                            }
                            None => {
                                // Record unresolved entries for further analysis
                                //pending.push((possible_indices.clone(), desc.clone()));
                                State::Unsolved((possible_indices, desc))
                            }
                        }
                    }

                    State::Solved(_) => entry,
                }
            })
            .collect(); // This is a placeholder

        // If no progress was made and there are pending items, attempt conflict resolution
        if !changed {
            res = res
                .into_iter()
                .map(|entry| match entry {
                    State::Solved(_) => entry,
                    State::Unsolved((possible_indices, desc)) => {
                        // Try to resolve conflicts by finding the least-burdened family
                        let candidate = possible_indices
                            .iter()
                            .filter_map(|&index| {
                                queues
                                    .get(index as usize)
                                    .map(|family| (index, family.queue_count))
                            })
                            .min_by_key(|&(_, available)| available);

                        if let Some((index, _)) = candidate {
                            if let Some(family) = queues.get_mut(index as usize) {
                                if family.queue_count > 0 {
                                    family.queue_count -= 1;
                                    changed = true;
                                    State::Solved((index, desc))
                                } else {
                                    State::Unsolved((possible_indices, desc))
                                }
                            } else {
                                State::Unsolved((possible_indices, desc))
                            }
                        } else {
                            State::Unsolved((possible_indices, desc))
                        }
                    }
                })
                .collect();
        }
    }

    // TODO: this does not account for the same queue family is assigned to multiple descriptions,
    // which is an issue since it causes multiple DeviceQueueCreateInfo's as well as incorrect
    // priority settings.
    if res.iter().any(|x| matches!(x, State::Unsolved(_))) {
        let unsolved_desc = res
            .into_iter()
            .filter_map(|x| match x {
                State::Unsolved((_, desc)) => Some(desc),
                _ => None,
            })
            .next()
            .unwrap();

        Err(error::InitError::QueueDescriptionCouldNotBeFilled(
            unsolved_desc.description.clone(),
        ))
    } else {
        Ok(SolvedQueueLayout {
            queue_family_buffer: queues,
            solved_queues: res
                .into_iter()
                .filter_map(|x| match x {
                    State::Solved((family, desc)) => Some((family, desc)),
                    _ => None,
                })
                .collect(),
        })
    }
}
