use crate::instance::Instance;
use crate::types::ExtensionProperties;
use crate::types::Layer;
use infrastructure::Promise;

use super::error;
use super::surface;
use super::types::ExtensionName;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct QueueDescription {
    flags: ash::vk::QueueFlags,
    count: u8,
    priorities: Vec<f32>,
}

#[derive(Debug)]
pub struct Queue {
    inner: ash::vk::Queue,
    capabilities: ash::vk::QueueFlags,
}

type QueuePromise = Promise<Rc<Queue>, QueueDescription>;
impl Queue {
    pub fn new_promise(desc: QueueDescription) -> QueuePromise {
        Promise::new(desc)
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

/// Order the devices.
/// Returning `None` means that the device is not suitable.
/// Otherwise higher the number, higher the priority.
/// Errors are not propagated back, if it does not match the predicate make sure to log it as an
/// error!
struct DeviceOrdering(fn(&mut PhysicalDeviceProperties) -> Option<u32>);

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
    physical_device_creation_info: PhysicalDeviceCreationInfo,
    extensions: Vec<ExtensionName>,
    layers: Vec<Layer>,
    /// After using `Device::new` these promises will be filled out.
    queues: Vec<Promise<Rc<Queue>, QueueDescription>>,
    surfaces: Vec<surface::Surface>,
    /// If you can work with missing extensions, overdefine this variable
    /// set it to change variables in your domain and handle it there.
    order: DeviceOrdering,
}

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
struct QueueSupportData<'a> {
    queue_datas: Vec<ash::vk::QueueFamilyProperties>,
    /// The indices are for the previous array.
    descriptions: Vec<Support<Vec<Indices>, &'a QueuePromise>>,
}
struct PhysicalDeviceProperties<'a> {
    /// List of requested items.
    extensions: Vec<Support<ExtensionProperties, ExtensionName>>,
    queues: QueueSupportData<'a>,
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
    pub fn get_init_details<'a>(
        instance: &Arc<Instance>,
        physical_device: &ash::vk::PhysicalDevice,
        extensions: &[ExtensionName],
        queues: &'a [Promise<Rc<Queue>, QueueDescription>],
    ) -> Result<PhysicalDeviceProperties<'a>, error::Error> {
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
                        true => Support::Unsupported(desc),
                        false => Support::Supported((matching_families, desc)),
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
}
impl Device {
    pub fn new(
        instance: &Arc<Instance>,
        info: DeviceCreationInfo,
    ) -> Result<Arc<Self>, error::Error> {
        let physical_devices = unsafe { instance.raw.enumerate_physical_devices() }
            .map_err(|_| error::Error::EnumeratePhysicalDevicesFailed)?;
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
            physical_device.ok_or(error::Error::SuitablePhysicalDeviceNotFound)?;

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
                Rc::new(Queue {
                    inner: queue,
                    capabilities: desc.description.flags,
                })
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
}

fn to_create_info<'q>(
    family: &ash::vk::QueueFamilyProperties,
    desc: &'q QueueDescription,
) -> ash::vk::DeviceQueueCreateInfo<'q> {
    ash::vk::DeviceQueueCreateInfo::default()
        .queue_family_index(family.queue_flags.as_raw())
        .queue_priorities(&desc.priorities)
}

struct SolvedQueueLayout<'a> {
    queue_family_buffer: Vec<ash::vk::QueueFamilyProperties>,
    solved_queues: Vec<(u16, &'a QueuePromise)>,
}
fn solve_queues(inp: QueueSupportData<'_>) -> Result<SolvedQueueLayout<'_>, error::Error> {
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

    enum State<'a> {
        Unsolved((Vec<u16>, &'a QueuePromise)),
        Solved((u16, &'a QueuePromise)),
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

        Err(error::Error::QueueDescriptionCouldNotBeFilled(
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
