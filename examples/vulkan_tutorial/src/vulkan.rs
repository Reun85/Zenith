//! Handle RAII for vulkan objects.
// Fields are deleted in the order of definition.

// TODO: move most functions under Device
// Instead of creating multiple single command buffers. Queue them together.
use crate::log;
use smallvec::SmallVec;
const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub struct VulkanApp {
    renderer: Renderer,
    window: Window,
}

pub struct Renderer {
    image_sampler: Sampler,
    image: DeviceImage,
    vertex_buffer: DeviceBuffer,
    index_buffer: DeviceBuffer,
    frames_in_flight: FramesInFlight,
    descriptor_pool: ash::vk::DescriptorPool,
    command_pool: CommandPool,
    frame_buffers: Vec<FrameBuffer>,
    render_pass: RenderPass,
    pipeline: Pipeline,
    swapchain: SwapChain,
    surface: Surface,
    device: Device,
    instance: Instance,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::debug!("Dropping Renderer");
        // Just to be sure
        unsafe { self.device.device.device_wait_idle().unwrap() };
        unsafe {
            self.frames_in_flight.frame_infos.iter().for_each(|x| {
                self.device
                    .destroy_semaphore(x.sync_objects.render_finished_semaphore, None);
                self.device
                    .destroy_semaphore(x.sync_objects.image_available_semaphore, None);
                self.device
                    .destroy_fence(x.sync_objects.in_flight_fence, None);
                self.device.destroy_buffer(x.uniform_buffer.0.buffer, None);
                self.device.free_memory(x.uniform_buffer.0.memory, None);
                self.device
                    .free_command_buffers(self.command_pool.pool, &[x.command_buffer.buffer]);
            });
        }
        unsafe {
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
        unsafe {
            self.device
                .destroy_sampler(self.image_sampler.sampler, None);
            self.device.destroy_image_view(self.image.view, None);
            self.device.destroy_image(self.image.image, None);
            self.device.free_memory(self.image.memory, None);
        }

        unsafe {
            self.device.destroy_buffer(*self.index_buffer, None);
            self.device.free_memory(self.index_buffer.memory, None);
        }
        unsafe {
            self.device.destroy_buffer(*self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer.memory, None);
        }
        unsafe {
            self.device
                .destroy_command_pool(self.command_pool.pool, None)
        };
        self.frame_buffers.iter().for_each(|x| unsafe {
            self.device.device.destroy_framebuffer(**x, None);
        });
        self.swapchain.image_views.iter().for_each(|x| unsafe {
            self.device.device.destroy_image_view(*x, None);
        });
        unsafe {
            self.pipeline
                .set_layout
                .iter()
                .for_each(|x| self.device.destroy_descriptor_set_layout(*x, None));
            self.device
                .device
                .destroy_pipeline(self.pipeline.pipeline, None);
            self.device
                .device
                .destroy_pipeline_layout(self.pipeline.pipeline_layout, None);
            self.device
                .destroy_render_pass(self.render_pass.render_pass, None);
        }
        log::debug!("Dropping swapchain");
        unsafe {
            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.swapchain, None);
        };
        log::debug!("Dropped swapchain");
    }
}

#[derive(smart_default::SmartDefault)]
struct FramesInFlight {
    current_frame: usize,
    frame_infos: SmallVec<[FrameInFlightData; MAX_FRAMES_IN_FLIGHT]>,
}

struct FrameInFlightData {
    sync_objects: SyncObjects,
    command_buffer: CommandBuffer,
    uniform_buffer: (DeviceBuffer, *mut std::ffi::c_void),
    descriptor_set: ash::vk::DescriptorSet,
}
struct FrameInfo<'a> {
    data: &'a FrameInFlightData,
    image_index: u32,
    suboptimal: bool,
}

impl FramesInFlight {
    pub fn new(
        sync_objs: smallvec::SmallVec<[SyncObjects; MAX_FRAMES_IN_FLIGHT]>,
        command_buffers: smallvec::SmallVec<[CommandBuffer; MAX_FRAMES_IN_FLIGHT]>,
        uniform_buffs: smallvec::SmallVec<
            [(DeviceBuffer, *mut std::ffi::c_void); MAX_FRAMES_IN_FLIGHT],
        >,
        descriptor_sets: smallvec::SmallVec<[ash::vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT]>,
    ) -> FramesInFlight {
        let frames = sync_objs
            .into_iter()
            .zip(command_buffers)
            .zip(uniform_buffs)
            .zip(descriptor_sets)
            .map(
                |(((sync_objects, command_buffer), uniform_buffer), descriptor_set)| {
                    FrameInFlightData {
                        sync_objects,
                        descriptor_set,
                        command_buffer,
                        uniform_buffer,
                    }
                },
            )
            .collect();
        FramesInFlight {
            frame_infos: frames,
            current_frame: 0,
        }
    }
    pub fn next<'a>(&'a mut self, device: &Device, swapchain: &SwapChain) -> FrameInfo<'a> {
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        let frame_info = &mut self.frame_infos[self.current_frame];
        unsafe {
            device
                .wait_for_fences(&[frame_info.sync_objects.in_flight_fence], true, u64::MAX)
                .unwrap();
            device
                .reset_fences(&[frame_info.sync_objects.in_flight_fence])
                .unwrap();
        }
        let (image_index, suboptimal) = unsafe {
            swapchain
                .swapchain_loader
                .acquire_next_image(
                    swapchain.swapchain,
                    u64::MAX,
                    frame_info.sync_objects.image_available_semaphore,
                    ash::vk::Fence::null(),
                )
                .unwrap()
        };
        FrameInfo {
            data: frame_info,
            image_index,
            suboptimal,
        }
    }
    //     pub fn next<'a>(&'a mut self,device:&Device,swapchain:&SwapChain) -> (&'a mut SyncObjects, &'a mut CommandBuffer, &'a mut FrameBuffer, u32, &'a mut (DeviceBuffer, *mut std::ffi::c_void), bool){
    //
    //         self.current_frame = (self.current_frame+1) % MAX_FRAMES_IN_FLIGHT;
    //
    //         unsafe {
    //             device
    //                 .wait_for_fences(
    //                     &[self.sync_objects[self.current_frame].in_flight_fence],
    //                     true,
    //                     u64::MAX,
    //                 )
    //                 .unwrap();
    //             device
    //                 .reset_fences(&[self.sync_objects[self.current_frame].in_flight_fence])
    //                 .unwrap();
    //         }
    //             let (image_index,suboptimal) = unsafe {
    //                 swapchain
    //                 .swapchain_loader
    //                 .acquire_next_image(
    //                     swapchain.swapchain,
    //                     u64::MAX,
    //                     self.sync_objects[self.current_frame].image_available_semaphore,
    //                     ash::vk::Fence::null(),
    //                 )
    //                 .unwrap() };
    //         return (&mut self.sync_objects[self.current_frame],&mut self.command_buffers[self.current_frame],&mut self.frame_buffers[image_index as usize],image_index,&mut self.uniform_buffers[self.current_frame],suboptimal)
    //
    //
    //     }
}
impl Renderer {
    pub fn draw_frame(&mut self) -> bool {
        let FrameInfo {
            data:
                FrameInFlightData {
                    sync_objects,
                    command_buffer,
                    uniform_buffer: (_, uniform_map),
                    descriptor_set,
                },
            image_index,
            suboptimal,
        } = self.frames_in_flight.next(&self.device, &self.swapchain);
        let frame_buffer = &self.frame_buffers[image_index as usize];
        unsafe {
            self.device
                .reset_command_buffer(**command_buffer, CommandBufferResetFlags::empty())
                .unwrap()
        };

        command_buffer
            .record(
                &self.device,
                &self.swapchain,
                &self.render_pass,
                frame_buffer,
                &self.pipeline,
                &self.vertex_buffer,
                &self.index_buffer,
                *descriptor_set,
            )
            .unwrap();

        let command_buffer = [command_buffer.buffer];

        let ubo = create_uniform_buffer_from_time((
            self.swapchain.extent.width,
            self.swapchain.extent.height,
        ));
        unsafe {
            ubo.copy_to_memory_address(*uniform_map);
        }

        let semaphore = [sync_objects.image_available_semaphore];
        let finished_signal = [sync_objects.render_finished_semaphore];
        let submit_info = ash::vk::SubmitInfo::builder()
            .wait_semaphores(&semaphore)
            .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffer)
            .signal_semaphores(&finished_signal);
        // Commit to the queue
        unsafe {
            self.device
                .queue_submit(
                    self.device.graphics_queue,
                    &[submit_info.build()],
                    sync_objects.in_flight_fence,
                )
                .unwrap()
        };

        let swapchain_khr = [self.swapchain.swapchain];
        let image_indices = [image_index];
        let present_info = ash::vk::PresentInfoKHR::builder()
            .wait_semaphores(&finished_signal)
            .swapchains(&swapchain_khr)
            .image_indices(&image_indices);
        let suboptimal2 = unsafe {
            self.swapchain
                .swapchain_loader
                .queue_present(self.device.present_queue, &present_info)
                .unwrap_or(false)
        };
        suboptimal2 || suboptimal
    }
    pub fn recreate_swapchain(&mut self, window: &winit::window::Window) -> anyhow::Result<()> {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }

        self.frame_buffers.iter().for_each(|x| unsafe {
            self.device.device.destroy_framebuffer(**x, None);
        });
        self.swapchain.image_views.iter().for_each(|x| unsafe {
            self.device.device.destroy_image_view(*x, None);
        });
        unsafe {
            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.swapchain, None)
        };

        let size = (window.inner_size().width, window.inner_size().height);
        let swapchain_support =
            query_swapchain_support(&self.surface, self.device.physical_device, size).unwrap();
        self.swapchain =
            self.instance
                .create_swapchain(&self.surface, swapchain_support, &self.device)?;

        self.frame_buffers =
            self.instance
                .create_frame_buffers(&self.device, &self.swapchain, &self.render_pass)?;
        Ok(())
    }
}

fn create_uniform_buffer_from_time(screen_size: (u32, u32)) -> UniformBufferObject {
    static mut START_TIME: Option<std::time::Instant> = None;
    let start = unsafe {
        match START_TIME {
            Some(x) => x,
            None => {
                START_TIME = Some(std::time::Instant::now());
                START_TIME.unwrap()
            }
        }
    };

    let time = std::time::Instant::now();
    let dur = time.duration_since(start);
    let time = dur.as_secs_f32();

    UniformBufferObject {
        model: cgmath::Matrix4::from_angle_z(cgmath::Deg(time * 90.0)),
        view: cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(0.01, 0.0, 4.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
        ),
        proj: cgmath::perspective(
            cgmath::Deg::from(cgmath::Rad(std::f32::consts::PI / 4.0)),
            screen_size.0 as f32 / screen_size.1 as f32,
            0.1,
            10.0,
        ),
        ..Default::default()
    }
}

struct FPSCounter {
    count: u32,
    time: std::time::Instant,
}
impl FPSCounter {
    fn new() -> Self {
        Self {
            count: 0,
            time: std::time::Instant::now(),
        }
    }
    fn test(&mut self) -> &Self {
        self.count += 1;
        if self.time.elapsed().as_secs() >= 1 {
            log::info!("FPS: {}", self.count);
            self.count = 0;
            self.time = std::time::Instant::now();
        }
        self
    }
}

pub type RunReturnType = i32;
impl VulkanApp {
    pub fn new() -> anyhow::Result<Self> {
        let _span = log::debug_span!("Creating App").entered();
        let library = VulkanLibrary::new()?;
        log::trace!("Created library");

        let window = Window::new()?;
        log::trace!("Created window");
        let instance = Self::create_instance(&window, &library)?;
        log::trace!("Created instance");
        let surface = library.create_surface(&instance, &window)?;
        log::trace!("Created surface");
        let size = window.window.inner_size();
        let size = (size.width, size.height);
        let (device, swapchainsupport) = instance.create_best_device(&surface, size)?;
        log::trace!("Created device");
        let swapchain = instance.create_swapchain(&surface, swapchainsupport, &device)?;
        log::trace!("Created swapchain");
        let render_pass = instance.create_render_pass(&device, &swapchain)?;
        log::trace!("Created render_pass");
        let pipeline = instance.create_pipeline(&device, &swapchain, &render_pass)?;
        log::trace!("Created pipeline");
        let frame_buffers = instance.create_frame_buffers(&device, &swapchain, &render_pass)?;
        log::trace!("Created framebuffers");
        let (command_pool, command_buffers) = instance.create_command_pool(&device)?;
        log::trace!("Created command pool and buffer");
        let sync_objects = instance.create_sync_objects(&device)?;
        log::trace!("Created sync objects");
        let vertex_buffer = instance.create_buffer_with_data(
            &device,
            &command_pool,
            ash::vk::BufferUsageFlags::VERTEX_BUFFER,
            &VERTICES,
        )?;
        log::trace!("Created vertex buffer");
        let index_buffer = instance.create_buffer_with_data(
            &device,
            &command_pool,
            ash::vk::BufferUsageFlags::INDEX_BUFFER,
            &INDICES,
        )?;
        log::trace!("Created index buffer");
        let uniform_buffers = instance.create_uniform_buffer(&device)?;
        log::trace!("Created uniform buffers");

        let image = instance.create_texture_image_from_path(
            &device,
            &command_pool,
            "/home/reun/all/dev/zenith/examples/vulkan_tutorial/src/assets/textures/statue.jpg"
                .into(),
        )?;
        log::trace!("Created Texture");
        let image_sampler = instance.create_image_sampler(&device, &image)?;
        log::trace!("Created Image Sampler");
        let descriptor_pool = instance.create_descriptor_pool(&device)?;
        log::trace!("Created descriptor pool");
        let descriptor_sets = instance.create_descriptor_sets(
            &device,
            &descriptor_pool,
            &uniform_buffers,
            &image_sampler,
            &image,
            &pipeline,
        )?;
        log::trace!("Created descriptor sets");

        Ok(Self {
            window,
            renderer: Renderer {
                image_sampler,
                image,
                index_buffer,
                vertex_buffer,
                instance,
                device,
                surface,
                swapchain,
                pipeline,
                render_pass,
                command_pool,
                frame_buffers,
                descriptor_pool,
                frames_in_flight: FramesInFlight::new(
                    sync_objects,
                    command_buffers,
                    uniform_buffers,
                    descriptor_sets,
                ),
            },
        })
    }

    fn create_instance(
        window: &Window,
        library: &VulkanLibrary,
    ) -> Result<Instance, anyhow::Error> {
        let require_extensions = {
            let mut res = window.get_required_extensions()?;

            if cfg!(any(target_os = "macos", target_os = "ios")) {
                res.push(ash::vk::KhrPortabilityEnumerationFn::name().into());
                // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
                res.push(ash::vk::KhrGetPhysicalDeviceProperties2Fn::name().into());
            }
            res
        };
        let optional_extensions = vec![ash::extensions::ext::DebugUtils::name()]
            .into_iter()
            .map(|x| x.into())
            .collect();
        let optional_extensions =
            library.check_if_extensions_are_supported(&require_extensions, optional_extensions)?;
        let enabled_extensions = require_extensions
            .into_iter()
            .chain(optional_extensions)
            .collect::<Vec<_>>();
        log::trace!("Optional extensions {:?}", enabled_extensions);
        // TODO: reset this to original impl
        // let layers = if cfg!(debug_assertions) {
        //     vec![Layer::VALIDATIONLAYER]
        // } else {
        //     vec![]
        // };
        let layers = vec![Layer::VALIDATIONLAYER];
        let validation_layers = library.filter_available_validation_layers(layers);
        let instance = library.create_instance(InstanceCreateInfo {
            application_name: "Example",
            enabled_layers: validation_layers,
            enabled_extensions,
            ..Default::default()
        })?;
        Ok(instance)
    }

    pub fn run(&mut self) -> anyhow::Result<RunReturnType> {
        let _span = log::info_span!("Running App").entered();
        use winit::event::*;
        use winit::event_loop::ControlFlow;
        let mut dirty: bool = false;
        let mut fps = FPSCounter::new();

        let run_return = winit::platform::run_return::EventLoopExtRunReturn::run_return(
            &mut self.window.event_loop,
            |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::WindowEvent {
                        event:
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    Event::WindowEvent {
                        window_id: _,
                        event: WindowEvent::Resized(_),
                    } => {
                        dirty = true;
                    }
                    Event::MainEventsCleared => {
                        if self.window.window.inner_size() != winit::dpi::PhysicalSize::new(0, 0) {
                            if dirty {
                                log::debug!(
                                    "Resized window to {:?}",
                                    self.window.window.inner_size()
                                );
                                self.renderer
                                    .recreate_swapchain(&self.window.window)
                                    .unwrap();
                            }
                            fps.test();
                            dirty = self.renderer.draw_frame();
                        }
                    }

                    _ => (),
                }
            },
        );
        unsafe { self.renderer.device.device_wait_idle().unwrap() };
        Ok(run_return)
    }
}

use ash::vk::{
    CommandBufferResetFlags, ImageViewCreateInfo, SemaphoreCreateInfo, ShaderModuleCreateInfo,
};
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    fmt::Debug,
};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

#[derive(derive_more::Deref)]
pub struct Window {
    pub event_loop: winit::event_loop::EventLoop<()>,
    #[deref]
    pub window: winit::window::Window,
}

#[derive(Debug, derive_more::From, derive_more::Into, derive_more::AsRef, derive_more::AsMut)]
pub struct Extension<'a>(&'a CStr);

impl Window {
    pub fn new() -> anyhow::Result<Self> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Example")
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .build(&event_loop)?;
        Ok(Self { event_loop, window })
    }
    pub fn get_required_extensions(&self) -> anyhow::Result<Vec<Extension<'static>>> {
        let surface_extensions =
            ash_window::enumerate_required_extensions(self.window.raw_display_handle())?;
        Ok(surface_extensions
            .iter()
            .map(|ext| unsafe { CStr::from_ptr(*ext) }.into())
            .collect::<Vec<_>>())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        log::debug!("Dropping window");
    }
}

/// Statically linked vulkan library at compile time.
pub struct VulkanLibrary {
    pub entry: ash::Entry,
}

const VULKAN_API_VERSION: u32 = ash::vk::make_api_version(0, 1, 3, 250);
const ENGINE_VERSION: u32 = ash::vk::make_api_version(0, 0, 0, 1);

const ENGINE_NAME: &CStr = cstr::cstr!("");
#[derive(Debug, smart_default::SmartDefault)]
pub struct InstanceCreateInfo<'a> {
    #[default = "Example"]
    pub application_name: &'a str,
    #[default(vec![])]
    pub enabled_extensions: Vec<Extension<'a>>,
    #[default({
        if cfg!(any(target_os = "macos", target_os = "ios")) {
            ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            ash::vk::InstanceCreateFlags::default()
        }
    })]
    pub flags: ash::vk::InstanceCreateFlags,
    #[default(vec![])]
    pub enabled_layers: Vec<Layer<'a>>,
}

#[derive(Debug, derive_more::From, derive_more::Into, derive_more::AsRef, derive_more::AsMut)]
pub struct Layer<'a>(&'a CStr);

impl Layer<'_> {
    pub const VALIDATIONLAYER: Layer<'static> =
        Layer(unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") });
}

#[derive(derive_more::Deref)]
pub struct Surface {
    #[deref]
    pub surface: ash::vk::SurfaceKHR,
    pub surface_loader: ash::extensions::khr::Surface,
}

impl Drop for Surface {
    fn drop(&mut self) {
        log::debug!("Dropping Surface");
        unsafe { self.surface_loader.destroy_surface(self.surface, None) };
        log::debug!("Dropped Surface");
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

impl<T: ash::vk::Handle> Debug for HandleWrapper<T> {
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

#[derive(derive_more::Deref)]
pub struct Device {
    #[deref]
    device: ash::Device,
    physical_device: ash::vk::PhysicalDevice,
    graphics_queue: ash::vk::Queue,
    present_queue: ash::vk::Queue,
    graphic_family_index: u32,
    present_family_index: u32,
    // NOTE: This could be circumvented by using by creating a god instance, that holds reference
    // to everything related to vulkan, and manually drops everything.
}

impl Device {
    pub fn begin_single_time_command_buffer(
        &self,
        command_pool: &CommandPool,
    ) -> anyhow::Result<CommandBuffer> {
        let alloc_info = ash::vk::CommandBufferAllocateInfo::builder()
            .command_pool(**command_pool)
            .level(ash::vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { self.allocate_command_buffers(&alloc_info) }?[0];
        let begin_info = ash::vk::CommandBufferBeginInfo::builder()
            .flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
        }?;
        Ok(CommandBuffer {
            buffer: command_buffer,
        })
    }
    pub fn end_single_time_command_buffer(
        &self,
        command_buffer: CommandBuffer,
        command_pool: &CommandPool,
    ) -> anyhow::Result<()> {
        unsafe { self.device.end_command_buffer(command_buffer.buffer) }?;
        let binding = [command_buffer.buffer];
        let submit_info = ash::vk::SubmitInfo::builder().command_buffers(&binding);
        unsafe {
            self.queue_submit(
                self.graphics_queue,
                &[submit_info.build()],
                ash::vk::Fence::null(),
            )
        }?;
        unsafe { self.device.queue_wait_idle(self.graphics_queue) }?;
        unsafe { self.free_command_buffers(**command_pool, &[command_buffer.buffer]) };
        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        log::debug!("Dropping device");

        unsafe {
            self.device.destroy_device(None);
        }
        log::debug!("Dropped device");
    }
}

impl VulkanLibrary {
    pub fn new() -> anyhow::Result<Self> {
        let entry = ash::Entry::linked();
        Ok(Self { entry })
    }

    pub fn create_surface(&self, instance: &Instance, window: &Window) -> anyhow::Result<Surface> {
        let windowinner = &window.window;
        let surface = unsafe {
            // platform independent
            ash_window::create_surface(
                &self.entry,
                &instance.inner,
                windowinner.raw_display_handle(),
                windowinner.raw_window_handle(),
                None,
            )?
        };
        Ok(Surface {
            surface,
            surface_loader: ash::extensions::khr::Surface::new(&self.entry, &instance.inner),
        })
    }

    fn messenger_create_info() -> ash::vk::DebugUtilsMessengerCreateInfoEXT {
        ash::vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            // NOTE: Scary
            // --Could probably pass an Arc?-- This is a terrible idea, it would mess up the
            // reference counting.
            //.user_data(user_data)
            .build()
    }
    pub fn create_instance(&self, info: InstanceCreateInfo) -> anyhow::Result<Instance> {
        let app_name = CString::new(info.application_name)?;
        let app_info = ash::vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .api_version(VULKAN_API_VERSION)
            .engine_version(ENGINE_VERSION)
            .engine_name(ENGINE_NAME);

        let enabled_extensions = info
            .enabled_extensions
            .into_iter()
            .map(|ext| ext.0.as_ptr())
            .collect::<Vec<_>>();

        let enabled_layer_names = info
            .enabled_layers
            .into_iter()
            .map(|x| x.0.as_ptr())
            .collect::<Vec<_>>();

        let mut debug_info_creation_info = Self::messenger_create_info();
        let mut create_info = ash::vk::InstanceCreateInfo::builder()
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layer_names)
            .flags(info.flags)
            .application_info(&app_info);
        // if cfg!(debug_assertions) {
        //     create_info = create_info.push_next(&mut debug_info_creation_info);
        // }
        // TODO: check if this is correct?
        create_info = create_info.push_next(&mut debug_info_creation_info);
        let instance = unsafe { self.entry.create_instance(&create_info, None)? };

        let debug_creation_info2 = Self::messenger_create_info();
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&self.entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_creation_info2, None)
                .unwrap()
        };

        Ok(Instance {
            inner: instance,
            allocation_callbacks: None,
            _app_name: app_name,
            _debug_utils: debug_utils_loader,
            _debug_messenger: debug_call_back,
        })
    }

    /// Ensures that all required and optional extensions are supported by the vulkan implementation. Throws an error if any of the required extensions are not supported.
    /// Optionally returns a list of supported optional extensions.
    pub(crate) fn check_if_extensions_are_supported<'b>(
        &self,
        required_extensions: &[Extension<'_>],
        optional_extensions: Vec<Extension<'b>>,
    ) -> Result<Vec<Extension<'b>>, ash::vk::Result> {
        let res = self
            .entry
            // TODO: what is this none?
            .enumerate_instance_extension_properties(None)
            .unwrap();
        let available_extensions = res
            .iter()
            .map(|ext| unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) })
            .collect::<Vec<_>>();
        let all_required_met = required_extensions.iter().fold(true, |acc, ext| {
            if !acc {
                false
            } else {
                let x = available_extensions.iter().any(|av| *av == ext.0);
                #[cfg(debug_assertions)]
                if !x {
                    log::error!("Required extension {:?} is not available", ext);
                }
                x
            }
        });
        if all_required_met {
            Ok(optional_extensions
                .into_iter()
                .filter(|ext| available_extensions.iter().any(|av| *av == ext.0))
                .collect::<Vec<_>>())
        } else {
            Err(ash::vk::Result::ERROR_EXTENSION_NOT_PRESENT)
        }
    }

    pub(crate) fn filter_available_validation_layers<'a>(
        &self,
        validation_layers: Vec<Layer<'a>>,
    ) -> Vec<Layer<'a>> {
        let res = self.entry.enumerate_instance_layer_properties().unwrap();
        let available_validation_layers = res
            .iter()
            .map(|layers| unsafe { CStr::from_ptr(layers.layer_name.as_ptr()) })
            .collect::<Vec<_>>();
        validation_layers
            .into_iter()
            .filter(|layer| {
                let x = available_validation_layers
                    .iter()
                    .any(|layer2| *layer2 == layer.0);

                if !x {
                    log::warn!("Validation layer {:?} is not available", layer);
                }
                x
            })
            .collect()
    }
}

#[derive(derive_more::Deref)]
pub struct Instance {
    #[deref]
    pub inner: ash::Instance,
    pub allocation_callbacks: Option<ash::vk::AllocationCallbacks>,
    _app_name: CString,
    _debug_utils: ash::extensions::ext::DebugUtils,
    _debug_messenger: ash::vk::DebugUtilsMessengerEXT,
}
#[derive(derive_more::Deref)]
pub struct SwapChain {
    #[deref]
    swapchain: ash::vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
    _images: Vec<ash::vk::Image>,
    image_views: Vec<ash::vk::ImageView>,
    surface_format: ash::vk::SurfaceFormatKHR,
    extent: ash::vk::Extent2D,
}

impl std::fmt::Debug for SwapChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapChain")
            .field("swapchain", &self.swapchain)
            .field("_images", &self._images)
            .field("image_views", &self.image_views)
            .field("surface_format", &self.surface_format)
            .field("extent", &self.extent)
            .finish()
    }
}

#[derive(derive_more::Deref)]
pub struct Pipeline {
    #[deref]
    pipeline: ash::vk::Pipeline,
    pipeline_layout: ash::vk::PipelineLayout,
    set_layout: [ash::vk::DescriptorSetLayout; 1],
}
impl Drop for Pipeline {
    fn drop(&mut self) {
        log::debug!("Dropping pipeline");
        log::debug!("Dropped pipeline");
    }
}

extern crate glsl_to_spirv_macro;
glsl_to_spirv_macro::shader! {
    root: "src/shaders",
    shaders:[
        {
                ty: "frag",
                path: "f.frag",
                name: frag
            },{
                name: vert,
                ty: "vertex",
                path: "v.vert",
        }
    ]
}
impl Drop for Instance {
    fn drop(&mut self) {
        log::debug!("Dropping instance");
        unsafe {
            self._debug_utils
                .destroy_debug_utils_messenger(self._debug_messenger, None)
        };

        unsafe {
            self.inner
                .destroy_instance(self.allocation_callbacks.as_ref())
        };
        log::debug!("Dropped instance");
    }
}
impl Instance {
    pub fn create_pipeline(
        &self,
        device: &Device,
        swapchain: &SwapChain,
        render_pass: &RenderPass,
    ) -> anyhow::Result<Pipeline> {
        let fragment_code = frag::load_words();
        let vertex_code = vert::load_words();

        let fragment = unsafe {
            let builder = ShaderModuleCreateInfo::builder().code(fragment_code);
            device.device.create_shader_module(&builder, None)
        }
        .unwrap();
        let vertex = unsafe {
            let builder = ShaderModuleCreateInfo::builder().code(vertex_code);
            device.device.create_shader_module(&builder, None)
        }
        .unwrap();

        const ENTRY_POINT: &CStr = cstr::cstr!("main");
        let fragment_stage = ash::vk::PipelineShaderStageCreateInfo::builder()
            .stage(ash::vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(ENTRY_POINT)
            .build();
        let vertex_stage = ash::vk::PipelineShaderStageCreateInfo::builder()
            .stage(ash::vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(ENTRY_POINT)
            .build();

        let stages = [fragment_stage, vertex_stage];

        // Currently there is no input.
        let binding_desc = [Vertex::get_binding_description()];
        let attribute_desc = Vertex::get_attribute_descriptions();
        let vert_input = ash::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_desc)
            .vertex_attribute_descriptions(&attribute_desc)
            .build();

        let assembly_info = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_info = [ash::vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build()];
        let scissors = [ash::vk::Rect2D::builder()
            .offset(ash::vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent)
            .build()];
        let viewport_state = ash::vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewport_info)
            .scissors(&scissors);

        let dynamic_states = [
            ash::vk::DynamicState::VIEWPORT,
            ash::vk::DynamicState::SCISSOR,
        ];

        let dynamic_state_info = ash::vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states)
            .build();
        let rasterizer = ash::vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(ash::vk::PolygonMode::FILL) // Point or Line
            .line_width(1.0)
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);
        let sampling = ash::vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);
        // depth and stencil is null
        let color_blend_attachment = [ash::vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(ash::vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(ash::vk::BlendFactor::ONE)
            .dst_color_blend_factor(ash::vk::BlendFactor::ZERO)
            .color_blend_op(ash::vk::BlendOp::ADD)
            .src_alpha_blend_factor(ash::vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(ash::vk::BlendFactor::ZERO)
            .alpha_blend_op(ash::vk::BlendOp::ADD)
            .build()];

        let color_blend_info = ash::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachment);

        let binded_desc = [
            UniformBufferObject::get_descriptor_set_layout_binding(),
            Sampler2D::get_descriptor_set_layout_binding(),
        ];
        let desc_info = ash::vk::DescriptorSetLayoutCreateInfo::builder().bindings(&binded_desc);
        let set_layout =
            [unsafe { device.device.create_descriptor_set_layout(&desc_info, None) }.unwrap()];

        let pipeline_layout = {
            let create_info = ash::vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layout);
            unsafe { device.device.create_pipeline_layout(&create_info, None) }?
        };
        let pipeline = unsafe {
            let create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
                .stages(&stages)
                .vertex_input_state(&vert_input)
                .input_assembly_state(&assembly_info)
                .viewport_state(&viewport_state)
                .dynamic_state(&dynamic_state_info)
                .rasterization_state(&rasterizer)
                .multisample_state(&sampling)
                .color_blend_state(&color_blend_info)
                .layout(pipeline_layout)
                .render_pass(render_pass.render_pass)
                .subpass(0)
                .base_pipeline_index(-1);

            let create_info = [create_info.build()];
            device.device.create_graphics_pipelines(
                ash::vk::PipelineCache::null(),
                &create_info,
                None,
            )
        }
        .unwrap()[0];
        unsafe {
            device.destroy_shader_module(fragment, None);
            device.destroy_shader_module(vertex, None);
        }
        Ok(Pipeline {
            pipeline,
            pipeline_layout,
            set_layout,
        })
    }
    pub fn create_swapchain(
        &self,
        surface: &Surface,
        details: SwapChainSupport,
        device: &Device,
    ) -> anyhow::Result<SwapChain> {
        let image_count = (details.capabilities.min_image_count + 1).max(2);
        let max_image_count = if details.capabilities.max_image_count == 0 {
            u32::MAX
        } else {
            details.capabilities.max_image_count
        };
        let image_count = image_count.min(max_image_count);
        let (image_sharing_mode, queue_family_indices) =
            if device.graphic_family_index != device.present_family_index {
                (
                    ash::vk::SharingMode::CONCURRENT,
                    vec![device.graphic_family_index, device.present_family_index],
                )
            } else {
                (ash::vk::SharingMode::EXCLUSIVE, vec![])
            };
        let create_info = ash::vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
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

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&self.inner, &device.device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

        let image_views = swapchain_images
            .iter()
            .map(|x| {
                self.create_image_view(device, x, details.format.format)
                    .unwrap()
            })
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
    pub fn create_best_device(
        &self,
        surface: &Surface,
        window_inner_size: (u32, u32),
    ) -> anyhow::Result<(Device, SwapChainSupport)> {
        let device_extensions = vec![ash::extensions::khr::Swapchain::name()];
        let (physical_dev, _, swap_chain_support, (graph_index, pres_index)) = {
            let physical_devices = unsafe { self.inner.enumerate_physical_devices() }?;
            let devices_and_properties = physical_devices.into_iter().map(|physical_device| {
                let properties =
                    unsafe { self.inner.get_physical_device_properties(physical_device) };
                let features = unsafe { self.inner.get_physical_device_features(physical_device) };
                (physical_device, properties, features)
            });
            let with_shader_support = devices_and_properties
                // TODO: is this actually shaders, or just geo shaders?
                .filter(|(_, _, features)| {
                    features.geometry_shader == ash::vk::Bool32::from(true)
                        && features.sampler_anisotropy == ash::vk::Bool32::from(true)
                });
            let with_graphics_queue_support = with_shader_support
                // Graphics queue also means transfer queue.
                .filter_map(|(physical_device, properties, _)| {
                    let queue_family_properties = unsafe {
                        self.inner
                            .get_physical_device_queue_family_properties(physical_device)
                    };
                    let queue_family_index = queue_family_properties.into_iter().enumerate().fold(
                        (None, None),
                        |(graph, pres), (i, x)| {
                            (
                                match graph {
                                    None => (x.queue_count > 0
                                        && x.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS))
                                    .then_some(i as u32),
                                    Some(_) => graph,
                                },
                                match pres {
                                    None => unsafe {
                                        surface
                                            .surface_loader
                                            .get_physical_device_surface_support(
                                                physical_device,
                                                i as u32,
                                                surface.surface,
                                            )
                                            .unwrap_or(false)
                                    }
                                    .then_some(i as u32),
                                    Some(_) => pres,
                                },
                            )
                        },
                    );
                    match queue_family_index {
                        (Some(graph), Some(pres)) => {
                            Some((physical_device, properties, (graph, pres)))
                        }
                        _ => None,
                    }
                });
            let with_swapchain_support =
                with_graphics_queue_support.filter(|(physical_device, _, _)| {
                    unsafe {
                        self.inner
                            .enumerate_device_extension_properties(*physical_device)
                    }
                    .map_or(false, |vec| {
                        device_extensions.iter().all(|ext| {
                            vec.iter().any(
                                |x| unsafe { CStr::from_ptr(x.extension_name.as_ptr()) } == *ext,
                            )
                        })
                    })
                });
            let with_image_support = with_swapchain_support
                // Check if image support is great:)
                .filter_map(|(physical_device, properties, (graph_index, pres_index))| {
                    let swap_chain_support =
                        query_swapchain_support(surface, physical_device, window_inner_size);
                    swap_chain_support
                        .map(|x| (physical_device, properties, x, (graph_index, pres_index)))
                });
            let best_device =
                with_image_support.max_by_key(|(_, properties, swap_chain_support, ..)| {
                    let format = &swap_chain_support.format;
                    let mut score = match properties.device_type {
                        ash::vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                        ash::vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                        ash::vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
                        _ => 0,
                    } * 1000;
                    score += properties.api_version;
                    score += match format.format {
                        ash::vk::Format::B8G8R8A8_SRGB => 10,
                        _ => 0,
                    };
                    score += match format.color_space {
                        ash::vk::ColorSpaceKHR::SRGB_NONLINEAR => 100,
                        _ => 0,
                    };
                    // Gives better resolution.
                    score += properties.limits.max_image_dimension2_d;
                    score
                });

            best_device.ok_or(anyhow::anyhow!("No physical devices found"))?
        };

        let queues = std::collections::HashSet::from([graph_index, pres_index]);

        let queue_create_info = queues
            .into_iter()
            .map(|queue_family_index| {
                ash::vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_family_index)
                    .queue_priorities(&[1.0])
                    .build()
            })
            .collect::<Vec<_>>();
        // We dont need device_features for now
        let binding = device_extensions
            .into_iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<_>>();
        let binding2 = ash::vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);
        let device_create_info = ash::vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&binding)
            .enabled_features(&binding2)
            .queue_create_infos(&queue_create_info);

        let device = unsafe {
            self.inner
                .create_device(physical_dev, &device_create_info, None)?
        };
        let graphic_queue = unsafe { device.get_device_queue(graph_index, 0) };
        let pres_queue = unsafe { device.get_device_queue(pres_index, 0) };

        let dev = Device {
            device,
            physical_device: physical_dev,
            graphics_queue: graphic_queue,
            present_queue: pres_queue,
            graphic_family_index: graph_index,
            present_family_index: pres_index,
        };
        Ok((dev, swap_chain_support))
    }

    fn create_render_pass(
        &self,
        device: &Device,
        swapchain: &SwapChain,
    ) -> anyhow::Result<RenderPass> {
        let attachment_info = ash::vk::AttachmentDescription::builder()
            .format(swapchain.surface_format.format)
            .samples(ash::vk::SampleCountFlags::TYPE_1)
            .load_op(ash::vk::AttachmentLoadOp::CLEAR)
            .store_op(ash::vk::AttachmentStoreOp::STORE)
            .stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(ash::vk::ImageLayout::UNDEFINED)
            .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR)
            .flags(ash::vk::AttachmentDescriptionFlags::empty());

        let attachment_ref = ash::vk::AttachmentReference::builder()
            .attachment(0)
            .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let binding = [attachment_ref];
        let subpass = ash::vk::SubpassDescription::builder()
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&binding);

        let dependency = ash::vk::SubpassDependency::builder()
            .src_subpass(ash::vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(ash::vk::AccessFlags::empty())
            .dst_stage_mask(ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let binding1 = [*attachment_info];
        let binding2 = [*dependency];
        let binding3 = [*subpass];
        let render_pass_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&binding1)
            .dependencies(&binding2)
            .subpasses(&binding3)
            .build();

        let render_pass = unsafe { device.device.create_render_pass(&render_pass_info, None) }?;

        Ok(RenderPass { render_pass })
    }

    fn create_frame_buffers(
        &self,
        device: &Device,
        swapchain: &SwapChain,
        render_pass: &RenderPass,
    ) -> anyhow::Result<Vec<FrameBuffer>> {
        let buffers = swapchain
            .image_views
            .iter()
            .map(|x| {
                let binding = [*x];
                let create = ash::vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.render_pass)
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(1)
                    .attachments(&binding)
                    .build();
                unsafe { device.create_framebuffer(&create, None) }.map(|x| x.into())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(buffers)
    }
    fn create_command_pool(
        &self,
        device: &Device,
    ) -> anyhow::Result<(CommandPool, SmallVec<[CommandBuffer; MAX_FRAMES_IN_FLIGHT]>)> {
        let create_info = ash::vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.graphic_family_index)
            .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let pool = unsafe { device.device.create_command_pool(&create_info, None) }?;
        let buffer = unsafe {
            let create_info = ash::vk::CommandBufferAllocateInfo::builder()
                .command_pool(pool)
                .level(ash::vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
            device.device.allocate_command_buffers(&create_info)
        }?;
        let pool = CommandPool { pool };
        let buffers = buffer
            .into_iter()
            .map(|buffer| CommandBuffer { buffer })
            .collect::<Vec<_>>();
        Ok((pool, buffers.into()))
    }
    fn create_sync_objects(
        &self,
        device: &Device,
    ) -> anyhow::Result<SmallVec<[SyncObjects; MAX_FRAMES_IN_FLIGHT]>> {
        let semaphore_create_info = SemaphoreCreateInfo::builder().build();
        let fence_create_info = ash::vk::FenceCreateInfo::builder()
            .flags(ash::vk::FenceCreateFlags::SIGNALED) // By default its signaled :3
            .build();
        Ok((0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let image_available_semaphore = unsafe {
                    device
                        .device
                        .create_semaphore(&semaphore_create_info, None)
                        .unwrap()
                };
                let render_finished_semaphore = unsafe {
                    device
                        .device
                        .create_semaphore(&semaphore_create_info, None)
                        .unwrap()
                };
                let in_flight_fence = unsafe {
                    device
                        .device
                        .create_fence(&fence_create_info, None)
                        .unwrap()
                };
                SyncObjects {
                    image_available_semaphore,
                    render_finished_semaphore,
                    in_flight_fence,
                }
            })
            .collect())
    }
    fn create_buffer(
        &self,
        device: &Device,
        buffer_size: u64,
        usage_flags: ash::vk::BufferUsageFlags,
        sharing_mode: ash::vk::SharingMode,
        memory_types: ash::vk::MemoryPropertyFlags,
    ) -> anyhow::Result<DeviceBuffer> {
        let (buffer, alloc_info) = self.get_memory_allocate_info(
            buffer_size,
            usage_flags,
            sharing_mode,
            device,
            memory_types,
        )?;
        let memory = unsafe { device.allocate_memory(&alloc_info, None) }?;
        unsafe { device.bind_buffer_memory(buffer, memory, 0) }?;

        Ok(DeviceBuffer { buffer, memory })
    }

    fn get_memory_allocate_info(
        &self,
        buffer_size: u64,
        usage_flags: ash::vk::BufferUsageFlags,
        sharing_mode: ash::vk::SharingMode,
        device: &Device,
        memory_types: ash::vk::MemoryPropertyFlags,
    ) -> Result<(ash::vk::Buffer, ash::vk::MemoryAllocateInfo), anyhow::Error> {
        let buffer_info = ash::vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(usage_flags)
            .sharing_mode(sharing_mode);
        let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;
        let memory_req = unsafe { device.get_buffer_memory_requirements(buffer) };
        let prop = unsafe { self.get_physical_device_memory_properties(device.physical_device) };
        let index = prop
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                memory_req.memory_type_bits & (1 << i) != 0
                    && memory_type.property_flags.contains(memory_types)
                //  memory_type.property_flags.contains(ash::vk::MemoryPropertyFlags::HOST_VISIBLE)
                // && memory_type.property_flags.contains(ash::vk::MemoryPropertyFlags::HOST_COHERENT)
            })
            .map(|(i, _)| i)
            .ok_or(anyhow::anyhow!("No suitable memory type found"))?;
        let alloc_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_req.size)
            .memory_type_index(index as u32)
            .build();
        Ok((buffer, alloc_info))
    }
    fn create_buffer_with_data<T>(
        &self,
        device: &Device,
        command_pool: &CommandPool,
        data_flags: ash::vk::BufferUsageFlags,
        data: &[T],
    ) -> anyhow::Result<DeviceBuffer> {
        // TODO: use a external library to allocate memory on the GPU once, then use that memory
        // with the offset variables, to use the buffer for all our buffers.
        let buffer_size = std::mem::size_of_val(data) as u64;

        let staging_buffer = self.create_buffer(
            device,
            buffer_size,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
            ash::vk::SharingMode::EXCLUSIVE,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer.memory,
                0,
                buffer_size,
                ash::vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut T, data.len());
            device.unmap_memory(staging_buffer.memory);
        }

        let final_buffer = self.create_buffer(
            device,
            buffer_size,
            data_flags | ash::vk::BufferUsageFlags::TRANSFER_DST,
            ash::vk::SharingMode::EXCLUSIVE,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        self.copy_buffer(
            device,
            command_pool,
            &staging_buffer,
            &final_buffer,
            buffer_size,
        )?;

        unsafe {
            device.destroy_buffer(staging_buffer.buffer, None);
            device.free_memory(staging_buffer.memory, None);
        }

        Ok(final_buffer)
    }
    fn create_uniform_buffer(
        &self,
        device: &Device,
    ) -> anyhow::Result<
        smallvec::SmallVec<[(DeviceBuffer, *mut std::ffi::c_void); MAX_FRAMES_IN_FLIGHT]>,
    > {
        let buffer_size = UniformBufferObject::size() as u64;
        (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let buffer = self.create_buffer(
                    device,
                    buffer_size,
                    ash::vk::BufferUsageFlags::UNIFORM_BUFFER,
                    ash::vk::SharingMode::EXCLUSIVE,
                    ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                        | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
                )?;
                let data_ptr = unsafe {
                    device.map_memory(
                        buffer.memory,
                        0,
                        buffer_size,
                        ash::vk::MemoryMapFlags::empty(),
                    )?
                };

                Ok((buffer, data_ptr))
            })
            .collect()
    }
    fn copy_buffer(
        &self,
        device: &Device,
        command_pool: &CommandPool,
        src: &DeviceBuffer,
        dst: &DeviceBuffer,
        size: ash::vk::DeviceSize,
    ) -> anyhow::Result<()> {
        let cmd = device.begin_single_time_command_buffer(command_pool)?;
        let copy_regions = [ash::vk::BufferCopy::builder().size(size).build()];
        unsafe { device.cmd_copy_buffer(*cmd, src.buffer, dst.buffer, &copy_regions) };
        device.end_single_time_command_buffer(cmd, command_pool)?;

        Ok(())
    }

    fn create_descriptor_pool(&self, device: &Device) -> anyhow::Result<ash::vk::DescriptorPool> {
        let pool_sizes = [
            ash::vk::DescriptorPoolSize::builder()
                .ty(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
                .build(),
            ash::vk::DescriptorPoolSize::builder()
                .ty(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
                .build(),
        ];
        let info = ash::vk::DescriptorPoolCreateInfo::builder()
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
            .pool_sizes(&pool_sizes)
            .build();

        let pool = unsafe { device.create_descriptor_pool(&info, None) }?;
        Ok(pool)
    }

    fn create_descriptor_sets(
        &self,
        device: &Device,
        descriptor_pool: &ash::vk::DescriptorPool,
        uniform_buffers: &SmallVec<[(DeviceBuffer, *mut std::ffi::c_void); 2]>,
        sampler: &Sampler,
        image_view: &DeviceImage,
        pipeline: &Pipeline,
    ) -> anyhow::Result<smallvec::SmallVec<[ash::vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT]>> {
        let layouts = [pipeline.set_layout[0]].repeat(MAX_FRAMES_IN_FLIGHT);
        let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(*descriptor_pool)
            .set_layouts(&layouts);
        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }?;
        sets.iter().enumerate().for_each(|(i, set)| {
            let buffer_info = ash::vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i].0.buffer)
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64)
                .build();
            let image_sampler_info = ash::vk::DescriptorImageInfo::builder()
                .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image_view.view)
                .sampler(sampler.sampler)
                .build();
            let binding = [buffer_info];
            let buffer_write = ash::vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&binding)
                .build();
            let binding2 = [image_sampler_info];
            let image_write = ash::vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&binding2)
                .build();
            unsafe { device.update_descriptor_sets(&[buffer_write, image_write], &[]) }
        });
        Ok(sets.into())
    }

    fn create_texture_image_from_path(
        &self,
        device: &Device,
        command_pool: &CommandPool,
        path: std::path::PathBuf,
    ) -> anyhow::Result<DeviceImage> {
        let image = image::open(path).unwrap();
        let image_data = image.into_rgba8();
        self.create_texture_image(device, command_pool, image_data)
    }

    fn transition_image_layout(
        &self,
        device: &Device,
        command_pool: &CommandPool,
        image: &DeviceImage,
        _format: ash::vk::Format,
        old_layout: ash::vk::ImageLayout,
        new_layout: ash::vk::ImageLayout,
    ) -> anyhow::Result<()> {
        let cmd = device.begin_single_time_command_buffer(command_pool)?;
        // let aspect_mask = if new_layout == ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        //     if format == ash::vk::Format::D32_SFLOAT_S8_UINT || format == ash::vk::Format::D24_UNORM_S8_UINT {
        //         ash::vk::ImageAspectFlags::DEPTH | ash::vk::ImageAspectFlags::STENCIL
        //     } else {
        //         ash::vk::ImageAspectFlags::DEPTH
        //     }
        // } else {
        //     ash::vk::ImageAspectFlags::COLOR
        // };
        let subresource_range = ash::vk::ImageSubresourceRange::builder()
            .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();
        // Essentially a barrier is a way to synchronize access to resources.
        // This may be sending resources with exclusive sharing access, or transitioning the
        // layout.
        let ((src_stage, src_mask), (dst_stage, dst_mask)) = match (old_layout, new_layout) {
            (ash::vk::ImageLayout::UNDEFINED, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                (
                    ash::vk::PipelineStageFlags::TOP_OF_PIPE,
                    ash::vk::AccessFlags::empty(),
                ),
                (
                    ash::vk::PipelineStageFlags::TRANSFER,
                    ash::vk::AccessFlags::TRANSFER_WRITE,
                ),
            ),
            (
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ) => (
                (
                    ash::vk::PipelineStageFlags::TRANSFER,
                    ash::vk::AccessFlags::TRANSFER_WRITE,
                ),
                (
                    ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
                    ash::vk::AccessFlags::SHADER_READ,
                ),
            ),
            _ => {
                log::error!("old: {:?} new:{:?}", old_layout, new_layout);
                unimplemented!("unsupported layout transition!")
            }
        };
        let barrier = ash::vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            // These have to be ignored not the default
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(src_mask)
            .dst_access_mask(dst_mask)
            .image(image.image)
            .subresource_range(subresource_range)
            .build();

        unsafe {
            device.cmd_pipeline_barrier(
                *cmd,
                src_stage,
                dst_stage,
                ash::vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };
        device.end_single_time_command_buffer(cmd, command_pool)?;
        Ok(())
    }
    fn create_texture_image(
        &self,
        device: &Device,
        command_pool: &CommandPool,
        image: image::RgbaImage,
    ) -> anyhow::Result<DeviceImage> {
        let (width, height) = image.dimensions();
        let image_data = image.into_raw();
        let image_size = (image_data.len() * std::mem::size_of::<u8>()) as u64;
        let staging_buffer = self.create_buffer(
            device,
            image_size,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
            ash::vk::SharingMode::EXCLUSIVE,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer.memory,
                0,
                image_size,
                ash::vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                image_data.as_ptr(),
                data_ptr as *mut u8,
                image_data.len(),
            );
            device.unmap_memory(staging_buffer.memory);
        }
        let image = self.create_image(
            device,
            width,
            height,
            ash::vk::Format::R8G8B8A8_SRGB,
            ash::vk::ImageTiling::OPTIMAL,
            ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
        )?;

        self.transition_image_layout(
            device,
            command_pool,
            &image,
            ash::vk::Format::R8G8B8A8_SRGB,
            ash::vk::ImageLayout::UNDEFINED,
            ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        self.copy_buffer_to_image(device, command_pool, &staging_buffer, &image, width, height)?;

        self.transition_image_layout(
            device,
            command_pool,
            &image,
            ash::vk::Format::R8G8B8A8_SRGB,
            ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        unsafe {
            device.destroy_buffer(staging_buffer.buffer, None);
            device.free_memory(staging_buffer.memory, None);
        }
        Ok(image)

        // texture_image
    }
    fn copy_buffer_to_image(
        &self,
        device: &Device,
        command_pool: &CommandPool,
        buffer: &DeviceBuffer,
        image: &DeviceImage,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let cmd = device.begin_single_time_command_buffer(command_pool)?;
        let region = ash::vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                ash::vk::ImageSubresourceLayers::builder()
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .image_offset(ash::vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(ash::vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .build();
        unsafe {
            device.cmd_copy_buffer_to_image(
                *cmd,
                buffer.buffer,
                image.image,
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            )
        };
        device.end_single_time_command_buffer(cmd, command_pool)?;
        Ok(())
    }

    fn create_image(
        &self,
        device: &Device,
        width: u32,
        height: u32,
        format: ash::vk::Format,
        tiling: ash::vk::ImageTiling,
        usage: ash::vk::ImageUsageFlags,
    ) -> Result<DeviceImage, anyhow::Error> {
        let create_info = ash::vk::ImageCreateInfo::builder()
            .extent(
                *ash::vk::Extent3D::builder()
                    .width(width)
                    .height(height)
                    .depth(1),
            )
            .array_layers(1)
            .mip_levels(1)
            .image_type(ash::vk::ImageType::TYPE_2D)
            .format(format)
            .tiling(tiling)
            .initial_layout(ash::vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            // NOTE: Could be used for sparse images.
            .samples(ash::vk::SampleCountFlags::TYPE_1);
        let image = unsafe { device.create_image(&create_info, None) }?;
        let memory_req = unsafe { device.get_image_memory_requirements(image) };
        let prop = unsafe { self.get_physical_device_memory_properties(device.physical_device) };
        let index = prop
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                memory_req.memory_type_bits & (1 << i) != 0
                    && memory_type
                        .property_flags
                        .contains(ash::vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })
            .map(|(i, _)| i)
            .ok_or(anyhow::anyhow!("No suitable memory type found"))?;
        let alloc_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_req.size)
            .memory_type_index(index as u32)
            .build();
        let texture_mem = unsafe { device.allocate_memory(&alloc_info, None) }?;
        unsafe { device.bind_image_memory(image, texture_mem, 0) }?;
        let image_view = self.create_image_view(device, &image, format)?;
        Ok(DeviceImage {
            image,
            memory: texture_mem,
            view: image_view,
        })
    }

    fn create_image_view(
        &self,
        device: &Device,
        image: &ash::vk::Image,
        format: ash::vk::Format,
    ) -> anyhow::Result<ash::vk::ImageView> {
        let create_info = ImageViewCreateInfo::builder()
            .image(*image)
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(ash::vk::ComponentMapping {
                r: ash::vk::ComponentSwizzle::IDENTITY,
                g: ash::vk::ComponentSwizzle::IDENTITY,
                b: ash::vk::ComponentSwizzle::IDENTITY,
                a: ash::vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(ash::vk::ImageSubresourceRange {
                aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let image_view = unsafe { device.create_image_view(&create_info, None) }?;
        Ok(image_view)
    }
    fn create_image_sampler(
        &self,
        device: &Device,
        _image: &DeviceImage,
    ) -> anyhow::Result<Sampler> {
        let properties = unsafe { self.get_physical_device_properties(device.physical_device) };

        let create_info = ash::vk::SamplerCreateInfo::builder()
            .mag_filter(ash::vk::Filter::LINEAR)
            .min_filter(ash::vk::Filter::LINEAR)
            .address_mode_u(ash::vk::SamplerAddressMode::REPEAT)
            .address_mode_v(ash::vk::SamplerAddressMode::REPEAT)
            .address_mode_w(ash::vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(properties.limits.max_sampler_anisotropy)
            .border_color(ash::vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(ash::vk::CompareOp::ALWAYS)
            .mipmap_mode(ash::vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);
        let sampler = unsafe { device.create_sampler(&create_info, None) }?;
        Ok(Sampler { sampler })
    }
}

#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct Sampler {
    #[deref]
    #[deref_mut]
    sampler: ash::vk::Sampler,
}
#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct DeviceImage {
    #[deref]
    #[deref_mut]
    image: ash::vk::Image,
    view: ash::vk::ImageView,
    memory: ash::vk::DeviceMemory,
}
#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct DeviceBuffer {
    #[deref]
    #[deref_mut]
    buffer: ash::vk::Buffer,
    memory: ash::vk::DeviceMemory,
}

fn query_swapchain_support(
    surface: &Surface,
    physical_device: ash::vk::PhysicalDevice,
    window_inner_size: (u32, u32),
) -> Option<SwapChainSupport> {
    let formats = unsafe {
        surface
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface.surface)
    }
    .unwrap();
    let present_modes = unsafe {
        surface
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface.surface)
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
            .get_physical_device_surface_capabilities(physical_device, surface.surface)
    }
    .unwrap();
    let mut extent: ash::vk::Extent2D = if present_capabilities.current_extent.width != u32::MAX {
        ash::vk::Extent2D::builder()
            .width(window_inner_size.0)
            .height(window_inner_size.1)
            .build()
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
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct CommandPool {
    #[deref]
    #[deref_mut]
    pool: ash::vk::CommandPool,
}
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct CommandBuffer {
    #[deref]
    #[deref_mut]
    buffer: ash::vk::CommandBuffer,
}
impl CommandBuffer {
    pub fn record(
        &self,
        device: &Device,
        swapchain: &SwapChain,
        render_pass: &RenderPass,
        frame_buffer: &FrameBuffer,
        pipeline: &Pipeline,
        vertex_buffer: &DeviceBuffer,
        index_buffer: &DeviceBuffer,
        descriptor_set: ash::vk::DescriptorSet,
    ) -> anyhow::Result<()> {
        let begin_info = ash::vk::CommandBufferBeginInfo::builder();
        unsafe {
            device
                .device
                .begin_command_buffer(self.buffer, &begin_info)
                .unwrap();
        }
        let clear_color = [ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let render_pass_info = ash::vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.render_pass)
            .framebuffer(**frame_buffer)
            .render_area(ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .clear_values(&clear_color);

        let viewport = ash::vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = ash::vk::Rect2D::builder()
            .offset(ash::vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent);
        unsafe {
            device.device.cmd_begin_render_pass(
                self.buffer,
                &render_pass_info,
                ash::vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                self.buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                **pipeline,
            );
            device.cmd_bind_descriptor_sets(
                self.buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );
            device.cmd_bind_vertex_buffers(self.buffer, 0, &[vertex_buffer.buffer], &[0]);
            device.cmd_bind_index_buffer(
                self.buffer,
                index_buffer.buffer,
                0,
                ash::vk::IndexType::UINT32,
            );
            device.cmd_set_viewport(self.buffer, 0, &[viewport.build()]);
            device.cmd_set_scissor(self.buffer, 0, &[scissor.build()]);
            device.cmd_draw_indexed(self.buffer, INDICES.len() as u32, 1, 0, 0, 0);
            device.device.cmd_end_render_pass(self.buffer);
            device.device.end_command_buffer(self.buffer)?;
        }
        Ok(())
    }
}
#[derive(derive_more::Deref, derive_more::DerefMut, derive_more::Into, derive_more::From)]
pub struct FrameBuffer {
    #[deref]
    buffer: ash::vk::Framebuffer,
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct RenderPass {
    #[deref]
    render_pass: ash::vk::RenderPass,
}

pub struct SyncObjects {
    image_available_semaphore: ash::vk::Semaphore,
    render_finished_semaphore: ash::vk::Semaphore,
    in_flight_fence: ash::vk::Fence,
}

/// A lambda function you can pass to vulkan.
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 {
    let callback_data = *p_callback_data;
    // let message_id_number = callback_data.message_id_number;

    let _message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
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
    use ash::vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
    let msg = format!("{}", message);
    // let msg = format!("{}: [{}] ({:?})", message_id_name, message, objects);
    match message_severity {
        Severity::ERROR => {
            log::error!(msg)
        }
        Severity::WARNING => {
            log::warn!(msg)
        }
        Severity::INFO => {
            log::info!(msg)
        }
        Severity::VERBOSE => {
            log::debug!(msg)
        }
        _ => {}
    }

    ash::vk::FALSE
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
    texture: [f32; 2],
}
pub type Index = u32;

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        texture: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
        texture: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
        texture: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        texture: [1.0, 1.0],
    },
];
const INDICES: [Index; 6] = [0, 1, 2, 2, 3, 0];

impl Vertex {
    pub const fn get_binding_description() -> ash::vk::VertexInputBindingDescription {
        ash::vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: ash::vk::VertexInputRate::VERTEX,
        }
    }
    pub const fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 3] {
        [
            ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: ash::vk::Format::R32G32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, position) as u32,
            },
            ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, color) as u32,
            },
            ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: ash::vk::Format::R32G32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, texture) as u32,
            },
        ]
    }
}

// This type matches
use cgmath::{Matrix4, SquareMatrix, Vector2};
#[repr(C, align(16))]
struct UniformBufferObject {
    eye: Vector2<f32>,
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl Default for UniformBufferObject {
    fn default() -> Self {
        Self {
            eye: Vector2::new(0.0, 0.0),
            model: Matrix4::identity(),
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
        }
    }
}
impl UniformBufferObject {
    unsafe fn copy_to_memory_address(&self, dst: *mut std::ffi::c_void) {
        std::ptr::copy_nonoverlapping(&self.eye, dst.offset(0) as *mut Vector2<f32>, 1);
        std::ptr::copy_nonoverlapping(&self.model, dst.offset(16) as *mut Matrix4<f32>, 1);
        std::ptr::copy_nonoverlapping(&self.view, dst.offset(80) as *mut Matrix4<f32>, 1);
        std::ptr::copy_nonoverlapping(&self.proj, dst.offset(144) as *mut Matrix4<f32>, 1);
    }

    const fn size() -> usize {
        std::mem::size_of::<cgmath::Vector4<f32>>() + std::mem::size_of::<Matrix4<f32>>() * 3
    }
}
impl UniformBufferObject {
    pub const fn get_descriptor_set_layout_binding() -> ash::vk::DescriptorSetLayoutBinding {
        ash::vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: ash::vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: ash::vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        }
    }
}

struct Sampler2D;

impl Sampler2D {
    pub const fn get_descriptor_set_layout_binding() -> ash::vk::DescriptorSetLayoutBinding {
        ash::vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: ash::vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        }
    }
}
