
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

pub use self::buffer::{SiegeBuffer, HostVisibleBuffer, DeviceLocalBuffer};
pub use self::image_wrap::ImageWrap;

use std::sync::Arc;

use dacite::core::{Instance, PhysicalDevice, PhysicalDeviceProperties,
                   PhysicalDeviceFeatures, Device, Queue, Extent2D,
                   ShaderModule, Rect2D, Viewport, Offset2D,
                   DescriptorPool, Semaphore, Fence};
use dacite::ext_debug_report::DebugReportCallbackExt;
use dacite::khr_surface::SurfaceKhr;
use winit::Window;

use self::setup::{Physical, QueueIndices};
use self::memory::{Memory, Lifetime};
use self::swapchain_data::SwapchainData;
use self::commander::Commander;
use self::resource_manager::ResourceManager;
use self::mesh::VulkanMesh;
use self::target_data::TargetData;
use super::vertex::*;
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

pub struct Renderer {
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
    queue_indices: QueueIndices,
    ph_feats: PhysicalDeviceFeatures,
    ph_props: PhysicalDeviceProperties,
    ph: PhysicalDevice,
    surface: SurfaceKhr,
    #[allow(dead_code)] // We don't use this directly, FFI uses it
    debug_callback: Option<DebugReportCallbackExt>,
    #[allow(dead_code)] // This must stay alive until we shut down
    instance: Instance,
    window: Arc<Window>,
    config: Arc<Config>,
}

impl Renderer {
    pub fn new(config: Arc<Config>, window: Arc<Window>)
               -> Result<Renderer>
    {
        use dacite::core::BufferUsageFlags;

        let instance = setup::setup_instance(&config, &window)?;

        let debug_callback = setup::setup_debug_callback(&config, &instance)?;

        let surface = setup::setup_surface(&window, &instance)?;

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

        let resource_manager = ResourceManager::new(
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

        Ok(Renderer {
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
            queue_indices: queue_indices,
            ph_feats: physical_device_features,
            ph_props: physical_device_properties,
            ph: physical_device,
            surface: surface,
            debug_callback: debug_callback,
            instance: instance,
            window: window,
            config: config
        })
    }

    pub fn run(&mut self) -> Result<()>
    {
        unimplemented!()
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
}
