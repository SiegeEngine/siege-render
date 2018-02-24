
mod setup;
mod memory;
mod buffer;
mod image_wrap;
mod surface_data;
mod swapchain_data;
mod commander;
mod mesh;
mod resource_manager;
mod target_data;
mod passes;
mod pipeline;
mod post;
mod blur;

pub use self::buffer::{SiegeBuffer, HostVisibleBuffer, DeviceLocalBuffer};
pub use self::image_wrap::ImageWrap;
pub use self::mesh::VulkanMesh;
pub use self::memory::Lifetime;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use dacite::core::{Instance, PhysicalDevice, Device, Queue, Extent2D,
                   ShaderModule, Rect2D, Viewport, Offset2D,
                   PhysicalDeviceProperties,
                   DescriptorPool, Semaphore, Fence,
                   BufferUsageFlags, DescriptorSetLayoutCreateInfo,
                   DescriptorSetLayout, DescriptorSet, Pipeline, PipelineLayout,
                   Timeout, SamplerCreateInfo, Sampler,
                   PipelineVertexInputStateCreateInfo, PrimitiveTopology,
                   CullModeFlags, FrontFace, ImageView};
use dacite::ext_debug_report::DebugReportCallbackExt;
use dacite::khr_surface::SurfaceKhr;
use winit::Window;

use self::setup::Physical;
use self::memory::Memory;
use self::swapchain_data::SwapchainData;
use self::commander::Commander;
use self::resource_manager::ResourceManager;
use self::target_data::TargetData;
use self::passes::{EarlyZPass, OpaquePass, TransparentPass,
                   BlurHPass, BlurVPass, PostPass, UiPass};
use self::post::PostGfx;
use self::blur::BlurGfx;
use super::vertex::*;
use super::plugin::Plugin;
use errors::*;
use config::Config;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum VulkanLogLevel {
    Error,
    Warning,
    PerformanceWarning,
    Information,
    Debug
}

// Passes that consumers of the library can plug into
pub enum Pass {
    EarlyZ,
    Opaque,
    Transparent,
    Ui
}

pub enum DepthHandling {
    None,
    Some(bool, bool) // test, write
}

pub struct Renderer {
    plugins: Vec<Box<Plugin>>,
    blur_gfx: BlurGfx,
    post_gfx: PostGfx,
    ui_pass: UiPass,
    post_pass: PostPass,
    blur_v_pass: BlurVPass,
    blur_h_pass: BlurHPass,
    transparent_pass: TransparentPass,
    opaque_pass: OpaquePass,
    early_z_pass: EarlyZPass,
    target_data: TargetData,
    graphics_fence: Fence,
    image_rendered: Semaphore,
    image_acquired: Semaphore,
    descriptor_pool: DescriptorPool,
    scissors: Vec<Rect2D>,
    viewports: Vec<Viewport>,
    staging_buffer: HostVisibleBuffer<u8>,
    resource_manager: ResourceManager,
    commander: Commander,
    present_queue: Queue,
    swapchain_data: SwapchainData,
    memory: Memory,
    device: Device,
    //queue_indices: QueueIndices,
    //ph_feats: PhysicalDeviceFeatures,
    ph_props: PhysicalDeviceProperties,
    ph: PhysicalDevice,
    surface: SurfaceKhr,
    #[allow(dead_code)] // We don't use this directly, FFI uses it
    debug_callback: Option<DebugReportCallbackExt>,
    #[allow(dead_code)] // This must stay alive until we shut down
    instance: Instance,
    shutdown: Arc<AtomicBool>,
    resized: Arc<AtomicBool>,
    window: Arc<Window>,
    config: Config,
}

impl Renderer {
    pub fn new(config: Config, window: Arc<Window>,
               resized: Arc<AtomicBool>,
               shutdown: Arc<AtomicBool>)
               -> Result<Renderer>
    {
        let instance = setup::setup_instance(&config, &window)?;

        let debug_callback = setup::setup_debug_callback(&config, &instance)?;

        let surface = setup::setup_surface(&window, &instance)?;

        #[allow(unused_variables)]
        let Physical {
            physical_device,
            physical_device_properties,
            physical_device_features,
            physical_device_memory_properties,
            queue_indices,
            device_extensions
        } = setup::find_suitable_device( &config, &instance, &surface)?;

        let device = setup::create_device(
            &physical_device, device_extensions, &queue_indices)?;

        let mut memory = Memory::new(physical_device_memory_properties,
                                     &physical_device_properties);

        let swapchain_data = SwapchainData::create(
            &physical_device, &device, &surface,
            Extent2D { width: config.width, height: config.height },
            &queue_indices, config.vsync)?;

        let present_queue = device.get_queue(queue_indices.present_family,
                                             queue_indices.present_index);

        let commander = Commander::new(
            &device, &queue_indices,
            swapchain_data.images.len() as u32)?;

        let mut resource_manager = ResourceManager::new(
            config.asset_path.clone());

        let staging_buffer = HostVisibleBuffer::new(
            &device, &mut memory,
            ::renderer::setup::requirements::MAX_GPU_UPLOAD,
            BufferUsageFlags::TRANSFER_SRC,
            Lifetime::Permanent, "Staging Buffer"
        )?;

        let viewports = vec![Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_data.extent.width as f32,
            height: swapchain_data.extent.height as f32,
            min_depth: if config.reversed_depth_buffer { 1.0 } else { 0.0 },
            max_depth: if config.reversed_depth_buffer { 0.0 } else { 1.0 },
        }];
        let scissors = vec![Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: swapchain_data.extent.clone(),
        }];

        let descriptor_pool = setup::get_descriptor_pool(&device, &config)?;

        let (image_acquired, image_rendered) = setup::get_semaphores(&device)?;

        let graphics_fence = setup::get_graphics_fence(&device)?;

        let target_data = TargetData::create(
            &device, &mut memory, &commander, swapchain_data.extent)?;

        let early_z_pass = EarlyZPass::new(
            &device, &target_data.depth_image, config.reversed_depth_buffer)?;
        let opaque_pass = OpaquePass::new(
            &device, &target_data.depth_image, &target_data.shading_image)?;
        let transparent_pass = TransparentPass::new(
            &device, &target_data.depth_image, &target_data.shading_image)?;
        let blur_h_pass = BlurHPass::new(
            &device, &target_data.shading_image, &target_data.blur_image)?;
        let blur_v_pass = BlurVPass::new(
            &device, &target_data.blur_image, &target_data.shading_image)?;
        let post_pass = PostPass::new(
            &device, &target_data.shading_image, &swapchain_data)?;
        let ui_pass = UiPass::new(
            &device, &swapchain_data)?;

        let post_gfx = PostGfx::new(&device, descriptor_pool.clone(),
                                    &target_data, &mut resource_manager,
                                    post_pass.render_pass.clone(),
                                    viewports[0].clone(), scissors[0].clone())?;

        let blur_gfx = BlurGfx::new(&device, descriptor_pool.clone(),
                                    &target_data, &mut resource_manager,
                                    blur_h_pass.render_pass.clone(),
                                    blur_v_pass.render_pass.clone(),
                                    viewports[0].clone(), scissors[0].clone())?;

        Ok(Renderer {
            plugins: Vec::new(),
            blur_gfx: blur_gfx,
            post_gfx: post_gfx,
            ui_pass: ui_pass,
            post_pass: post_pass,
            blur_v_pass: blur_v_pass,
            blur_h_pass: blur_h_pass,
            transparent_pass: transparent_pass,
            opaque_pass: opaque_pass,
            early_z_pass: early_z_pass,
            target_data: target_data,
            graphics_fence: graphics_fence,
            image_rendered: image_rendered,
            image_acquired: image_acquired,
            descriptor_pool: descriptor_pool,
            scissors: scissors,
            viewports: viewports,
            staging_buffer: staging_buffer,
            resource_manager: resource_manager,
            commander: commander,
            present_queue: present_queue,
            swapchain_data: swapchain_data,
            memory: memory,
            device: device,
            //queue_indices: queue_indices,
            //ph_feats: physical_device_features,
            ph_props: physical_device_properties,
            ph: physical_device,
            surface: surface,
            debug_callback: debug_callback,
            instance: instance,
            shutdown: shutdown,
            resized: resized,
            window: window,
            config: config
        })
    }

    pub fn load_shader(&mut self, name: &str) -> Result<ShaderModule>
    {
        self.resource_manager.load_shader(&self.device, name)
    }

    pub fn load_graybox_mesh(&mut self, name: &str) -> Result<VulkanMesh<GrayboxVertex>>
    {
        self.resource_manager.load_graybox_mesh(
            &self.device, &mut self.memory, &self.commander, &self.staging_buffer, name)
    }

    pub fn load_cubemap_mesh(&mut self, name: &str) -> Result<VulkanMesh<CubemapVertex>>
    {
        self.resource_manager.load_cubemap_mesh(
            &self.device, &mut self.memory, &self.commander, &self.staging_buffer, name)
    }

    pub fn load_texture(&mut self, name: &str) -> Result<ImageWrap>
    {
        self.resource_manager.load_texture(
            &self.device, &mut self.memory, &self.commander, &self.staging_buffer, name)
    }

    pub fn get_image_view(&self, image: &ImageWrap) -> Result<ImageView>
    {
        image.get_image_view(&self.device)
    }

    pub fn get_extent(&self) -> Extent2D {
        self.swapchain_data.extent
    }

    pub fn get_min_uniform_alignment(&self) -> usize {
        self.ph_props.limits.min_uniform_buffer_offset_alignment as usize
    }

    pub fn create_pipeline(&mut self,
                           desc_set_layouts: Vec<DescriptorSetLayout>,
                           vertex_shader: Option<&str>,
                           fragment_shader: Option<&str>,
                           vertex_type: Option<PipelineVertexInputStateCreateInfo>,
                           topology: PrimitiveTopology,
                           cull_mode: CullModeFlags,
                           front_face: FrontFace,
                           depth_handling: DepthHandling,
                           alpha_blend: bool,
                           pass: Pass)
                           -> Result<(PipelineLayout, Pipeline)>
    {
        let vs = match vertex_shader {
            Some(vs) => Some(self.load_shader(vs)?),
            None => None
        };
        let fs = match fragment_shader {
            Some(fs) => Some(self.load_shader(fs)?),
            None => None
        };
        pipeline::create(
            &self.device, self.viewports[0].clone(), self.scissors[0].clone(),
            self.config.reversed_depth_buffer,
            match pass {
                Pass::EarlyZ => self.early_z_pass.render_pass.clone(),
                Pass::Opaque => self.opaque_pass.render_pass.clone(),
                Pass::Transparent => self.transparent_pass.render_pass.clone(),
                Pass::Ui => self.ui_pass.render_pass.clone(),
            },
            desc_set_layouts, vs, fs,
            vertex_type, topology, cull_mode, front_face, depth_handling, alpha_blend)
    }

    pub fn create_sampler(&mut self,
                          create_info: SamplerCreateInfo)
                          -> Result<Sampler>
    {
        Ok(self.device.create_sampler(&create_info, None)?)
    }

    pub fn create_host_visible_buffer(
        &mut self, size: u64, usage: BufferUsageFlags,
        lifetime: Lifetime, reason: &str)
        -> Result<HostVisibleBuffer<u8>>
    {
        HostVisibleBuffer::new(
            &self.device, &mut self.memory,
            size, usage, lifetime, reason)
    }

    pub fn create_device_local_buffer<T: Copy>(
        &mut self, usage: BufferUsageFlags,
        lifetime: Lifetime, reason: &str, data: &[T])
        -> Result<DeviceLocalBuffer<T>>
    {
        DeviceLocalBuffer::new_uploaded(
            &self.device, &mut self.memory, &self.commander,
            &self.staging_buffer, data, usage,
            lifetime, reason)
    }

    pub fn create_descriptor_set(&mut self, create_info: DescriptorSetLayoutCreateInfo)
                                        -> Result<(DescriptorSetLayout, DescriptorSet)>
    {
        let layout = self.device.create_descriptor_set_layout(&create_info, None)?;

        use dacite::core::DescriptorSetAllocateInfo;
        let alloc_info = DescriptorSetAllocateInfo {
            descriptor_pool: self.descriptor_pool.clone(),
            set_layouts: vec![layout.clone()],
            chain: None,
        };
        let mut descriptor_sets = DescriptorPool::allocate_descriptor_sets(&alloc_info)?;
        let set = descriptor_sets.pop().unwrap();

        Ok((layout, set))
    }

    pub fn plugin(&mut self, plugin: Box<Plugin>)
    {
        self.plugins.push(plugin);
    }

    // This will hog the current thread and wont return until the renderer shuts down.
    pub fn run(&mut self) -> Result<()>
    {
        use dacite::core::Error::OutOfDateKhr;

        self.window.show();
        self.record_command_buffers()?;
        self.memory.log_usage();
        self.graphics_fence.wait_for(Timeout::Infinite)?;

        let mut frame_number: u64 = 0;
        let loop_throttle = Duration::new(
            0, 1_000_000_000 / self.config.fps_cap);
        let mut render_start: Instant;
        let mut render_end: Instant;
        let mut render_duration: Duration;
        let mut render_duration_sum: Duration = Duration::new(0,0);
        let mut report_time: Instant = Instant::now();

        loop {
            for plugin in &mut self.plugins {
                plugin.update()?;
                plugin.upload()?;
            }

            // Render a frame
            render_start = match self.render() {
                Err(e) => {
                    if let &ErrorKind::Dacite(OutOfDateKhr) = e.kind() {
                        // Rebuild the swapchain if Vulkan complains that it is out of date.
                        // This is typical on linux.
                        self.rebuild()?;
                        self.graphics_fence.wait_for(Timeout::Infinite)?;
                        // Now we have rebuilt but we didn't render, so skip the rest of
                        // the loop and try to render again right away
                        continue;
                    } else {
                        return Err(e);
                    }
                },
                Ok(instant) => instant
            };

            frame_number += 1;

            // On windows (at least, perhaps also elsewhere), vulkan won't give us an
            // OutOfDateKhr error on a window resize.  But the window will remain black
            // after resizing.  We have to detect resizes and rebuild the swapchain.
            if self.resized.load(Ordering::Relaxed) {
                self.rebuild()?;
                self.resized.store(false, Ordering::Relaxed);
                self.graphics_fence.wait_for(Timeout::Infinite)?;
                continue;
            }

            // Wait until the GPU is idle.
            self.graphics_fence.wait_for(Timeout::Infinite)?;
            render_end = Instant::now();

            render_duration = render_end.duration_since(render_start);
            render_duration_sum += render_duration;

            // Throttle FPS
            if render_duration < loop_throttle {
                ::std::thread::sleep(loop_throttle - render_duration);
            }

            // FPS calculation every 500 frames
            if frame_number % 500 == 0 {
                let seconds_per_frame = duration_to_seconds(&render_duration_sum)/500.0;
                let fps = 500.0 / duration_to_seconds(&report_time.elapsed());
                trace!("{:>6.1} fps; {:>8.6} s/frame; {:>5.1}%",
                       fps, seconds_per_frame,
                       100.0 * seconds_per_frame / 0.016666667);

                // reset data
                report_time = Instant::now();
                render_duration_sum = Duration::new(0, 0);
            }

            // Shutdown when it is time to do so
            if self.shutdown.load(Ordering::Relaxed) {
                info!("Graphics is shutting down...");
                self.device.wait_idle()?;
                self.window.hide();
                return Ok(());
            }
        }
    }

    fn render(&mut self) -> Result<Instant>
    {
        use std::time::Duration;
        use dacite::core::{Timeout, SubmitInfo, PipelineStageFlags};
        use dacite::khr_swapchain::{AcquireNextImageResultKhr, PresentInfoKhr};

        // Get next image
        let next_image;
        loop {
            let next_image_res = self.swapchain_data.swapchain
                .acquire_next_image_khr(
                    Timeout::Some(Duration::from_millis(4000)),
                    Some(&self.image_acquired),
                    None)?;

            match next_image_res {
                AcquireNextImageResultKhr::Index(idx) |
                AcquireNextImageResultKhr::Suboptimal(idx) => {
                    next_image = idx;
                    break;
                },
                AcquireNextImageResultKhr::NotReady => {
                    ::std::thread::sleep(Duration::from_millis(100));
                    continue;
                },
                AcquireNextImageResultKhr::Timeout => {
                    return Err(ErrorKind::SwapchainTimeout.into())
                }
            }
        };

        // Submit command buffers
        let start = {
            let submit_infos = vec![
                SubmitInfo {
                    wait_semaphores: vec![self.image_acquired.clone()],
                    wait_dst_stage_mask: vec![PipelineStageFlags::TOP_OF_PIPE],
                    command_buffers: vec![self.commander.gfx_command_buffers[next_image].clone()],
                    signal_semaphores: vec![self.image_rendered.clone()],
                    chain: None,
                }
            ];
            self.graphics_fence.reset()?;
            self.commander.gfx_queue.submit(Some(&submit_infos), Some(&self.graphics_fence))?;
            Instant::now()
        };

        // Present this image once semaphore is available
        // The CPU is not stalled here, the graphics card will hold this until the semaphore
        // is signalled, and then do the presentation.
        {
            let mut present_info = PresentInfoKhr {
                wait_semaphores: vec![self.image_rendered.clone()],
                swapchains: vec![self.swapchain_data.swapchain.clone()],
                image_indices: vec![next_image as u32],
                results: None,
                chain: None,
            };

            self.present_queue.queue_present_khr(&mut present_info)?;
        }

        Ok(start)
    }

    fn record_command_buffers(&mut self) -> Result<()>
    {
        // NOTE: recording a command buffer is well known as one of the slower
        // parts of Vulkan, so this should not be done every frame.

        use dacite::core::{CommandBufferBeginInfo, CommandBufferUsageFlags,
                           CommandBufferResetFlags, ImageLayout,
                           AccessFlags, PipelineStageFlags, ImageAspectFlags,
                           OptionalMipLevels, OptionalArrayLayers,
                           ImageSubresourceRange};

        for (present_index, command_buffer) in
            self.commander.gfx_command_buffers.iter().enumerate()
        {
            // Not sure this is required - was working with out it.  Also, not sure
            // if releasing resources is the smartest plan either.
            command_buffer.reset(CommandBufferResetFlags::empty())?;

            let begin_info = CommandBufferBeginInfo {
                flags: CommandBufferUsageFlags::empty(),
                inheritance_info: None,
                chain: None,
            };
            command_buffer.begin(&begin_info)?;

            // Transition swapchain image to ColorAttachmentOptimal
            // (from whatever it was - usually it is PresentImageKhr, but the
            //  very first time it will be Undefined).
            self.swapchain_data.images[present_index].transition_layout(
                command_buffer.clone(),
                ImageLayout::Undefined, ImageLayout::ColorAttachmentOptimal,
                AccessFlags::HOST_READ, AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::HOST, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: OptionalMipLevels::MipLevels(1),
                    base_array_layer: 0,
                    layer_count: OptionalArrayLayers::ArrayLayers(1),
                }
            )?;

            // Bind viewports and scissors
            command_buffer.set_viewport(0, &self.viewports);
            command_buffer.set_scissor(0, &self.scissors);

            self.target_data.transition_for_earlyz(command_buffer.clone())?;

            // Early Z pass
            {
                self.early_z_pass.record_entry(command_buffer.clone())?;

                for plugin in &self.plugins {
                    // NOTE: Try to draw front to back
                    plugin.record_earlyz(command_buffer.clone());
                }

                self.early_z_pass.record_exit(command_buffer.clone())?;
            }

            self.target_data.transition_for_opaque(command_buffer.clone())?;

            // Opaque pass
            {
                self.opaque_pass.record_entry(command_buffer.clone())?;

                for plugin in &self.plugins {
                    // Draw all geometry with opaque pipelines
                    // Draw in any order - it makes no difference,
                    // except for far-plane items (each overwrites the last)

                    // NOTE: Try to draw front to back
                    plugin.record_opaque(command_buffer.clone());
                }

                self.opaque_pass.record_exit(command_buffer.clone())?;
            }

            self.target_data.transition_for_transparent(command_buffer.clone())?;

            // Transparent pass
            {
                self.transparent_pass.record_entry(command_buffer.clone())?;

                for plugin in &self.plugins {
                    plugin.record_transparent(command_buffer.clone());
                }

                self.transparent_pass.record_exit(command_buffer.clone())?;
            }

            self.target_data.transition_for_blurh(command_buffer.clone())?;

            // Blur/Bloom Filter/Horizontal pass
            {
                self.blur_h_pass.record_entry(command_buffer.clone())?;

                self.blur_gfx.record_blurh(command_buffer.clone());

                self.blur_h_pass.record_exit(command_buffer.clone())?;
            }

            self.target_data.transition_for_blurv(command_buffer.clone())?;

            // Blur/Bloom Vertical/Merge pass
            {
                self.blur_v_pass.record_entry(command_buffer.clone())?;

                self.blur_gfx.record_blurv(command_buffer.clone());

                self.blur_v_pass.record_exit(command_buffer.clone())?;
            }

            self.target_data.transition_for_post(command_buffer.clone())?;

            // Post pass
            {
                self.post_pass.record_entry(command_buffer.clone(),
                                            present_index)?;

                self.post_gfx.record(command_buffer.clone());

                self.post_pass.record_exit(command_buffer.clone())?;
            }

            self.target_data.transition_for_ui(command_buffer.clone())?;

            // Ui pass
            {
                self.ui_pass.record_entry(command_buffer.clone(),
                                          present_index)?;

                for plugin in &self.plugins {
                    plugin.record_ui(command_buffer.clone());
                }

                self.ui_pass.record_exit(command_buffer.clone())?;
            }

            // Transition swapchain image to PresentImageKhr
            self.swapchain_data.images[present_index].transition_layout(
                command_buffer.clone(),
                ImageLayout::ColorAttachmentOptimal, ImageLayout::PresentSrcKhr,
                AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::HOST_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::HOST,
                ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: OptionalMipLevels::MipLevels(1),
                    base_array_layer: 0,
                    layer_count: OptionalArrayLayers::ArrayLayers(1),
                }
            )?;

            command_buffer.end()?;
        }

        Ok(())
    }

    fn rebuild(&mut self) -> Result<()>
    {
        // Wait until the device is idle
        self.device.wait_idle()?;

        // Rebuild swapchain
        self.swapchain_data.rebuild(&self.ph, &self.device, &self.surface)?;

        // Rebuild the targets
        self.target_data.rebuild(&self.device, &mut self.memory, &self.commander,
                                 self.swapchain_data.extent)?;

        // Rebuild the passes
        self.early_z_pass.rebuild(&self.device,
                                  &self.target_data.depth_image)?;
        self.opaque_pass.rebuild(&self.device,
                                 &self.target_data.depth_image,
                                 &self.target_data.shading_image)?;
        self.transparent_pass.rebuild(&self.device,
                                      &self.target_data.depth_image,
                                      &self.target_data.shading_image)?;
        self.blur_h_pass.rebuild(&self.device,
                                 &self.target_data.shading_image,
                                 &self.target_data.blur_image)?;
        self.blur_v_pass.rebuild(&self.device,
                                 &self.target_data.blur_image,
                                 &self.target_data.shading_image)?;
        self.post_pass.rebuild(&self.device,
                               &self.target_data.shading_image,
                               &self.swapchain_data)?;
        self.ui_pass.rebuild(&self.device,
                             &self.swapchain_data)?;

        // Rebuild post, blur
        self.post_gfx.rebuild(&self.device, &self.target_data)?;
        self.blur_gfx.rebuild(&self.device, &self.target_data)?;

        // Update viewports and scissors
        self.viewports[0].width = self.swapchain_data.extent.width as f32;
        self.viewports[0].height = self.swapchain_data.extent.height as f32;
        self.scissors[0].extent = self.swapchain_data.extent;

        // Rebuild plugins
        for plugin in &mut self.plugins {
            plugin.rebuild(self.swapchain_data.extent)?;
        }

        // Re-record command buffers (the framebuffer image views are new, so we must)
        self.record_command_buffers()?;

        Ok(())
    }
}

fn duration_to_seconds(duration: &Duration) -> f32
{
    duration.as_secs() as f32 +
        duration.subsec_nanos() as f32 * 0.000_000_001
}
