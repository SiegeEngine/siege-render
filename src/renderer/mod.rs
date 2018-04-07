
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
mod shade;
mod post;
mod blur;
mod stats;

pub use self::buffer::{HostVisibleBuffer, DeviceLocalBuffer};
pub use self::image_wrap::ImageWrap;
pub use self::mesh::VulkanMesh;
pub use self::memory::Lifetime;
pub use self::post::Tonemapper;
pub use self::stats::{Timings, Stats};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use dacite::core::{Instance, PhysicalDevice, Device, Queue, Extent2D,
                   ShaderModule, Rect2D, Viewport, Offset2D,
                   DescriptorPool, Semaphore, Fence,
                   BufferUsageFlags, DescriptorSetLayoutCreateInfo,
                   DescriptorSetLayout, DescriptorSet, Pipeline, PipelineLayout,
                   Timeout, SamplerCreateInfo, Sampler,
                   PipelineVertexInputStateCreateInfo, PrimitiveTopology,
                   CullModeFlags, FrontFace, ImageView,
                   DescriptorSetAllocateInfo, DescriptorType, ShaderStageFlags,
                   WriteDescriptorSetElements, DescriptorSetLayoutBinding,
                   PhysicalDeviceFeatures, PhysicalDeviceProperties,
                   Format, BufferView, SpecializationInfo, QueryPool,
                   QueryPoolCreateInfo, QueryType, QueryPipelineStatisticFlags,
                   QueryResultFlags, PipelineStageFlagBits, QueryResult};
use dacite::ext_debug_report::DebugReportCallbackExt;
use dacite::khr_surface::SurfaceKhr;
use siege_math::{Vec4, Mat4};
use winit::Window;

use self::setup::Physical;
use self::memory::Memory;
use self::swapchain_data::SwapchainData;
use self::commander::Commander;
use self::resource_manager::ResourceManager;
use self::target_data::TargetData;
use self::passes::{GeometryPass, ShadingPass, TransparentPass,
                   BlurHPass, BlurVPass, PostPass, UiPass};
use self::shade::ShadeGfx;
use self::post::PostGfx;
use self::blur::BlurGfx;
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
    Geometry,
    Transparent,
    Ui
}

pub enum DepthHandling {
    None,
    Some(bool, bool) // test, write
}

pub enum BlendMode {
    Off,
    Alpha,
    Add
}

#[repr(u32)]
pub enum Timestamp {
    FullStart = 0,
    FullEnd = 1,
    GeometryStart = 2,
    GeometryEnd = 3,
    ShadingStart = 4,
    ShadingEnd = 5,
    TransparentStart = 6,
    TransparentEnd = 7,
    Blur1Start = 8,
    Blur1End = 9,
    Blur2Start = 10,
    Blur2End = 11,
    PostStart = 12,
    PostEnd = 13,
    UiStart = 14,
    UiEnd = 15,
}
const TS_QUERY_COUNT: u32 = 16;

// FIXME: Some settings the renderer is trying to pass to its shaders (and different ones
//          to different shaders).
//        Some settings clients are trying to adjust in the renderer
//        At some point we had a value the renderer was communicating back to the client
//        These are not all the same set.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub inv_projection: Mat4<f32>,
    pub dlight_directions: [Vec4<f32>; 2],
    pub dlight_irradiances: [Vec4<f32>; 2],
    pub bloom_strength: f32, // 0.65
    pub bloom_cliff: f32, // 0.7
    pub blur_level: f32, // 0.0
    pub ambient: f32,
    pub white_level: f32,
    pub tonemapper: Tonemapper,
}

pub struct Renderer {
    plugins: Vec<Box<Plugin>>,
    post_gfx: PostGfx,
    blur_gfx: BlurGfx,
    shade_gfx: ShadeGfx,
    params_desc_set: DescriptorSet,
    #[allow(dead_code)]
    params_desc_layout: DescriptorSetLayout,
    #[allow(dead_code)]
    params_ubo: HostVisibleBuffer,
    ui_pass: UiPass,
    post_pass: PostPass,
    blur_v_pass: BlurVPass,
    blur_h_pass: BlurHPass,
    transparent_pass: TransparentPass,
    shading_pass: ShadingPass,
    geometry_pass: GeometryPass,
    target_data: TargetData,
    timestamp_query_pool: QueryPool,
    rendered_fence: Fence,
    acquired_fence: Fence,
    image_rendered: Semaphore,
    image_acquired: Semaphore,
    descriptor_pool: DescriptorPool,
    scissors: Vec<Rect2D>,
    viewports: Vec<Viewport>,
    staging_buffer: HostVisibleBuffer,
    resource_manager: ResourceManager,
    commander: Commander,
    present_queue: Queue,
    swapchain_data: SwapchainData,
    memory: Memory,
    device: Device,
    //queue_indices: QueueIndices,
    ph_feats: PhysicalDeviceFeatures,
    ph_props: PhysicalDeviceProperties,
    ph: PhysicalDevice,
    surface: SurfaceKhr,
    #[allow(dead_code)] // We don't use this directly, FFI uses it
    debug_callback: Option<DebugReportCallbackExt>,
    #[allow(dead_code)] // This must stay alive until we shut down
    instance: Instance,
    shutdown: Arc<AtomicBool>,
    resized: Arc<AtomicBool>,
    stats: Stats,
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
                                     physical_device_properties.clone());

        let swapchain_data = SwapchainData::create(
            &physical_device, &device, &surface,
            Extent2D { width: config.width, height: config.height },
            &queue_indices)?;

        let present_queue = device.get_queue(queue_indices.present_family,
                                             queue_indices.present_index);

        let commander = Commander::new(
            &device, &queue_indices,
            swapchain_data.images.len() as u32)?;

        let resource_manager = ResourceManager::new(
            config.asset_path.clone());

        let staging_buffer = HostVisibleBuffer::new::<u8>(
            &device, &mut memory,
            ::renderer::setup::requirements::MAX_GPU_UPLOAD as usize,
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

        let acquired_fence = setup::get_graphics_fence(&device, false)?;
        let rendered_fence = setup::get_graphics_fence(&device, false)?;

        let timestamp_query_pool = device.create_query_pool(&QueryPoolCreateInfo {
            flags: Default::default(),
            query_type: QueryType::Timestamp,
            query_count: TS_QUERY_COUNT,
            pipeline_statistics: QueryPipelineStatisticFlags::empty(),
            chain: None,
        }, None)?;

        let target_data = TargetData::create(
            &device, &mut memory, &commander, swapchain_data.extent)?;

        let geometry_pass = GeometryPass::new(
            &device, &target_data.depth_image, &target_data.diffuse_image,
            &target_data.normals_image, &target_data.material_image,
            config.reversed_depth_buffer)?;
        let shading_pass = ShadingPass::new(
            &device, &target_data.depth_image, &target_data.diffuse_image,
            &target_data.normals_image, &target_data.material_image,
            &target_data.shading_image)?;
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

        let mut params_ubo = HostVisibleBuffer::new::<Params>(
            &device, &mut memory, 1,
            BufferUsageFlags::UNIFORM_BUFFER,
            Lifetime::Permanent,
            "Render Parameter Uniforms")?;

        // write initial data
        {
            let params = Params {
                inv_projection: Mat4::identity(),
                dlight_directions: [
                    Default::default(),
                    Default::default() ],
                dlight_irradiances: [
                    Default::default(),
                    Default::default() ],
                bloom_strength: 0.65,
                bloom_cliff: 0.7,
                blur_level: 0.0,
                ambient: 0.001,
                white_level: 0.1,
                tonemapper: Tonemapper::Reinhard,
            };
            params_ubo.write_one(&params, None)?;
        }

        let (params_desc_layout, params_desc_set) = {
            let layout = {
                let create_info = DescriptorSetLayoutCreateInfo {
                    flags: Default::default(),
                    bindings: vec![
                        DescriptorSetLayoutBinding {
                            binding: 0,
                            descriptor_type: DescriptorType::UniformBuffer,
                            descriptor_count: 1, // just one UBO
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: vec![],
                        },
                    ],
                    chain: None,
                };
                device.create_descriptor_set_layout(&create_info, None)?
            };

            let alloc_info = DescriptorSetAllocateInfo {
                descriptor_pool: descriptor_pool.clone(),
                set_layouts: vec![layout.clone()],
                chain: None,
            };
            let mut dsets = DescriptorPool::allocate_descriptor_sets(&alloc_info)?;
            let descriptor_set = dsets.pop().unwrap();

            use dacite::core::{OptionalDeviceSize, DescriptorBufferInfo,
                               WriteDescriptorSet};
            DescriptorSet::update(
                Some(&[
                    WriteDescriptorSet {
                        dst_set: descriptor_set.clone(),
                        dst_binding: 0,
                        dst_array_element: 0, // only have 1 element
                        descriptor_type: DescriptorType::UniformBuffer,
                        elements: WriteDescriptorSetElements::BufferInfo(
                            vec![
                                DescriptorBufferInfo {
                                    buffer: params_ubo.inner(),
                                    offset: 0,
                                    range: OptionalDeviceSize::Size(
                                        ::std::mem::size_of::<Params>() as u64
                                    ),
                                }
                            ]
                        ),
                        chain: None,
                    }
                ]),
                None
            );

            (layout, descriptor_set)
        };

        let shade_gfx = ShadeGfx::new(&device, descriptor_pool.clone(),
                                      &target_data,
                                      shading_pass.render_pass.clone(),
                                      viewports[0].clone(), scissors[0].clone(),
                                      params_desc_layout.clone(),
                                      config.reversed_depth_buffer)?;

        let blur_gfx = BlurGfx::new(&device, descriptor_pool.clone(),
                                    &target_data,
                                    blur_h_pass.render_pass.clone(),
                                    blur_v_pass.render_pass.clone(),
                                    viewports[0].clone(), scissors[0].clone(),
                                    params_desc_layout.clone())?;

        let post_gfx = PostGfx::new(&device, descriptor_pool.clone(),
                                    &target_data, post_pass.render_pass.clone(),
                                    viewports[0].clone(), scissors[0].clone(),
                                    config.display_luminance,
                                    params_desc_layout.clone(),
                                    swapchain_data.surface_data.needs_gamma)?;

        Ok(Renderer {
            plugins: Vec::new(),
            post_gfx: post_gfx,
            blur_gfx: blur_gfx,
            shade_gfx: shade_gfx,
            params_desc_set: params_desc_set,
            params_desc_layout: params_desc_layout,
            params_ubo: params_ubo,
            ui_pass: ui_pass,
            post_pass: post_pass,
            blur_v_pass: blur_v_pass,
            blur_h_pass: blur_h_pass,
            transparent_pass: transparent_pass,
            shading_pass: shading_pass,
            geometry_pass: geometry_pass,
            target_data: target_data,
            timestamp_query_pool: timestamp_query_pool,
            rendered_fence: rendered_fence,
            acquired_fence: acquired_fence,
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
            ph_feats: physical_device_features,
            ph_props: physical_device_properties,
            ph: physical_device,
            surface: surface,
            debug_callback: debug_callback,
            instance: instance,
            shutdown: shutdown,
            resized: resized,
            stats: Default::default(),
            window: window,
            config: config
        })
    }

    pub fn load_shader(&mut self, name: &str) -> Result<ShaderModule>
    {
        self.resource_manager.load_shader(&self.device, name)
    }

    pub fn load_mesh(&mut self, dir: &str, name: &str) -> Result<VulkanMesh>
    {
        self.resource_manager.load_mesh(
            &self.device, &mut self.memory, &self.commander,
            &mut self.staging_buffer, dir, name)
    }

    pub fn load_texture(&mut self, name: &str) -> Result<ImageWrap>
    {
        self.resource_manager.load_texture(
            &self.device, &mut self.memory, &self.commander,
            &mut self.staging_buffer, name)
    }

    pub fn load_buffer(&mut self,
                       usage: BufferUsageFlags,
                       name: &str) -> Result<DeviceLocalBuffer>
    {
        self.resource_manager.load_buffer(
            &self.device, &mut self.memory, &self.commander,
            &mut self.staging_buffer, usage, name)
    }

    pub fn get_asset_path(&self) -> PathBuf {
        self.config.asset_path.clone()
    }

    pub fn get_image_view(&self, image: &ImageWrap) -> Result<ImageView>
    {
        image.get_image_view(&self.device)
    }

    pub fn get_buffer_view(&self, buffer: &DeviceLocalBuffer, format: Format)
        -> Result<BufferView>
    {
        buffer.get_buffer_view(&self.device, format)
    }

    pub fn get_extent(&self) -> Extent2D {
        self.swapchain_data.extent
    }

    pub fn has_anisotrophy(&self) -> bool {
        self.ph_feats.sampler_anisotropy
    }

    pub fn max_anisotrophy(&self) -> f32 {
        if self.has_anisotrophy() {
            self.ph_props.limits.max_sampler_anisotropy
        } else {
            1.0
        }
    }

    pub fn create_pipeline(&mut self,
                           desc_set_layouts: Vec<DescriptorSetLayout>,
                           vertex_shader: Option<&str>,
                           vertex_shader_spec: Option<SpecializationInfo>,
                           fragment_shader: Option<&str>,
                           fragment_shader_spec: Option<SpecializationInfo>,
                           vertex_type: Option<PipelineVertexInputStateCreateInfo>,
                           topology: PrimitiveTopology,
                           cull_mode: CullModeFlags,
                           front_face: FrontFace,
                           depth_handling: DepthHandling,
                           blend: Vec<BlendMode>,
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
                Pass::Geometry => self.geometry_pass.render_pass.clone(),
                Pass::Transparent => self.transparent_pass.render_pass.clone(),
                Pass::Ui => self.ui_pass.render_pass.clone(),
            },
            desc_set_layouts, vs, vertex_shader_spec, fs, fragment_shader_spec,
            vertex_type, topology, cull_mode, front_face, depth_handling, blend)
    }

    pub fn create_sampler(&mut self,
                          create_info: SamplerCreateInfo)
                          -> Result<Sampler>
    {
        Ok(self.device.create_sampler(&create_info, None)?)
    }

    pub fn create_host_visible_buffer<T>(
        &mut self, count: usize, usage: BufferUsageFlags,
        lifetime: Lifetime, reason: &str)
        -> Result<HostVisibleBuffer>
    {
        HostVisibleBuffer::new::<T>(
            &self.device, &mut self.memory,
            count, usage, lifetime, reason)
    }

    pub fn create_device_local_buffer<T: Copy>(
        &mut self, usage: BufferUsageFlags,
        lifetime: Lifetime, reason: &str, data: &[T])
        -> Result<DeviceLocalBuffer>
    {
        DeviceLocalBuffer::new_uploaded::<T>(
            &self.device, &mut self.memory, &self.commander,
            &mut self.staging_buffer, data, usage,
            lifetime, reason)
    }

    pub fn get_stride<T>(&self, usage: BufferUsageFlags) -> usize
    {
        self.memory.stride(::std::mem::size_of::<T>(), Some(usage))
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

    pub fn set_params(&mut self, params: &Params) -> Result<()>
    {
        self.params_ubo.write_one::<Params>(&params, None)
    }

    // This will hog the current thread and wont return until the renderer shuts down.
    pub fn run(&mut self) -> Result<()>
    {
        use dacite::core::Error::OutOfDateKhr;

        self.window.show();
        self.record_command_buffers()?;
        self.memory.log_usage();

        let mut framenumber: u64 = 0;

        let loop_throttle = Duration::new(0, 1_000_000_000 / self.config.fps_cap);

        let mut timings_60 = Timings::new();
        let mut timings_600 = Timings::new();

        let mut last_loop_start: Instant;
        let mut loop_start: Instant = Instant::now();
        loop {
            last_loop_start = loop_start;
            loop_start = Instant::now();
            let looptime_1 = loop_start.duration_since(last_loop_start);

            {
                let params = self.params_ubo.as_ptr::<Params>().unwrap();
                for plugin in &mut self.plugins {
                    plugin.update(params, &self.stats)?;
                }
            }
            self.memory.flush()?;

            // Render a frame (issue the commands)
            match self.render() {
                Err(e) => {
                    if let &ErrorKind::Dacite(OutOfDateKhr) = e.kind() {
                        // Rebuild the swapchain if Vulkan complains that it is out of date.
                        // This is typical on linux.
                        self.rebuild()?;
                        // Now we have rebuilt but we didn't render, so skip the rest of
                        // the loop and try to render again right away
                        self.acquired_fence.wait_for(Timeout::Infinite)?;
                        self.rendered_fence.wait_for(Timeout::Infinite)?;
                        continue;
                    } else {
                        return Err(e);
                    }
                },
                Ok(instant) => instant
            };

            framenumber += 1;

            // On windows (at least, perhaps also elsewhere), vulkan won't give us an
            // OutOfDateKhr error on a window resize.  But the window will remain black
            // after resizing.  We have to detect resizes and rebuild the swapchain.
            if self.resized.load(Ordering::Relaxed) {
                self.rebuild()?;
                self.resized.store(false, Ordering::Relaxed);
                self.acquired_fence.wait_for(Timeout::Infinite)?;
                self.rendered_fence.wait_for(Timeout::Infinite)?;
                continue;
            }

            // Wait until the frame is rendered
            // FIXME: we might actually be ready to acquire another image
            // before the previous one gets rendered. So this waiting here
            // could potentially be a bottleneck. But we do it to get more
            // accurate timings (for now). FIXME.
            // Wait on the acquired fence, before trying to acquire another
            self.acquired_fence.wait_for(Timeout::Infinite)?;
            let cpuwait_start = Instant::now();
            self.rendered_fence.wait_for(Timeout::Infinite)?;
            let cpuwait_1 = cpuwait_start.elapsed();

            // Query render timings
            let timings_1 = {
                let mut results: [QueryResult; TS_QUERY_COUNT as usize]
                    = [QueryResult::U32(0); TS_QUERY_COUNT as usize];
                self.timestamp_query_pool.get_results(
                    0, // first query
                    TS_QUERY_COUNT, // query count
                    1, // stride (dacite takes this and multiplies by size of u32 or u64
                    QueryResultFlags::WAIT,
                    &mut results
                )?;
                Timings::one(
                    &looptime_1,
                    &cpuwait_1,
                    &results,
                    self.ph_props.limits.timestamp_period)
            };

            // Throttle FPS
            let elapsed = loop_start.elapsed();
            if elapsed < loop_throttle {
                ::std::thread::sleep(loop_throttle - elapsed);
            }

            // Shutdown when it is time to do so
            if self.shutdown.load(Ordering::Relaxed) {
                info!("Graphics is shutting down...");
                self.device.wait_idle()?;
                self.window.hide();
                return Ok(());
            }

            // Update statistics
            timings_60.accumulate(&timings_1);
            timings_600.accumulate(&timings_1);

            if framenumber % 600 == 0 {
                let pass = ::std::mem::replace(&mut timings_600, Timings::new());
                self.stats.update_600(pass);
            }
            if framenumber % 60 == 0 {
                let pass = ::std::mem::replace(&mut timings_60, Timings::new());
                self.stats.update_60(pass);
            }
        }
    }

    fn render(&mut self) -> Result<()>
    {
        use std::time::Duration;
        use dacite::core::{Timeout, SubmitInfo, PipelineStageFlags};
        use dacite::khr_swapchain::{AcquireNextImageResultKhr, PresentInfoKhr};

        // Get next image
        let next_image;
        loop {
            self.acquired_fence.reset()?;
            let next_image_res = self.swapchain_data.swapchain
                .acquire_next_image_khr(
                    Timeout::Some(Duration::from_millis(4_000)),
                    Some(&self.image_acquired),
                    Some(&self.acquired_fence))?;

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
                    error!("Swapchain image acquisition timed out (but we keep trying)");
                    ::std::thread::sleep(Duration::from_millis(100));
                    continue;
                    //return Err(ErrorKind::SwapchainTimeout.into())
                }
            }
        };

        // Submit command buffers
        let submit_infos = vec![
            SubmitInfo {
                wait_semaphores: vec![self.image_acquired.clone()],
                wait_dst_stage_mask: vec![PipelineStageFlags::TOP_OF_PIPE],
                command_buffers: vec![self.commander.gfx_command_buffers[next_image].clone()],
                signal_semaphores: vec![self.image_rendered.clone()],
                chain: None,
            }
        ];

        self.rendered_fence.reset()?;
        self.commander.gfx_queue.submit(Some(&submit_infos), Some(&self.rendered_fence))?;

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

        Ok(())
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

            command_buffer.reset_query_pool(&self.timestamp_query_pool, 0, TS_QUERY_COUNT);

            command_buffer.write_timestamp(
                PipelineStageFlagBits::TopOfPipe,
                &self.timestamp_query_pool,
                Timestamp::FullStart as u32);

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

            self.target_data.transition_for_geometry(command_buffer.clone())?;

            // Geometry pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::GeometryStart as u32);

                self.geometry_pass.record_entry(command_buffer.clone());

                for plugin in &self.plugins {
                    // NOTE: Try to draw front to back
                    plugin.record_geometry(command_buffer.clone());
                }

                self.geometry_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::GeometryEnd as u32);
            }

            self.target_data.transition_for_shading(command_buffer.clone())?;

            // Shading pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::ShadingStart as u32);

                self.shading_pass.record_entry(command_buffer.clone());

                self.shade_gfx.record(command_buffer.clone(),
                                      self.params_desc_set.clone());

                self.shading_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::ShadingEnd as u32);
            }

            self.target_data.transition_for_transparent(command_buffer.clone())?;

            // Transparent pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::TransparentStart as u32);

                self.transparent_pass.record_entry(command_buffer.clone());

                for plugin in &self.plugins {
                    plugin.record_transparent(command_buffer.clone());
                }

                self.transparent_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::TransparentEnd as u32);
            }

            self.target_data.transition_for_blurh(command_buffer.clone())?;

            // Blur/Bloom Filter/Horizontal pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::Blur1Start as u32);

                self.blur_h_pass.record_entry(command_buffer.clone());

                self.blur_gfx.record_blurh(command_buffer.clone(),
                                           self.params_desc_set.clone());

                self.blur_h_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::Blur1End as u32);
            }

            self.target_data.transition_for_blurv(command_buffer.clone())?;

            // Blur/Bloom Vertical/Merge pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::Blur2Start as u32);

                self.blur_v_pass.record_entry(command_buffer.clone());

                self.blur_gfx.record_blurv(command_buffer.clone(),
                                           self.params_desc_set.clone());

                self.blur_v_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::Blur2End as u32);
            }

            self.target_data.transition_for_post(command_buffer.clone())?;

            // Post pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::PostStart as u32);

                self.post_pass.record_entry(command_buffer.clone(),
                                            present_index);

                self.post_gfx.record(command_buffer.clone(),
                                     self.params_desc_set.clone());

                self.post_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::PostEnd as u32);
            }

            self.target_data.transition_for_ui(command_buffer.clone())?;

            // Ui pass
            {
                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::UiStart as u32);

                self.ui_pass.record_entry(command_buffer.clone(),
                                          present_index);

                for plugin in &self.plugins {
                    plugin.record_ui(command_buffer.clone());
                }

                self.ui_pass.record_exit(command_buffer.clone());

                command_buffer.write_timestamp(
                    PipelineStageFlagBits::TopOfPipe,
                    &self.timestamp_query_pool,
                    Timestamp::UiEnd as u32);
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

            command_buffer.write_timestamp(
                PipelineStageFlagBits::TopOfPipe,
                &self.timestamp_query_pool,
                Timestamp::FullEnd as u32);

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
        self.geometry_pass.rebuild(&self.device,
                                   &self.target_data.depth_image,
                                   &self.target_data.diffuse_image,
                                   &self.target_data.normals_image,
                                   &self.target_data.material_image)?;
        self.shading_pass.rebuild(&self.device,
                                 &self.target_data.depth_image,
                                 &self.target_data.diffuse_image,
                                 &self.target_data.normals_image,
                                 &self.target_data.material_image,
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
        self.shade_gfx.rebuild(&self.device, &self.target_data)?;
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
