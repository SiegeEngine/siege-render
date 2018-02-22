
mod setup;

mod memory;

mod image_wrap;

mod surface_data;

mod swapchain_data;

use std::sync::Arc;

use dacite::core::{Instance, PhysicalDevice, PhysicalDeviceProperties,
                   PhysicalDeviceFeatures, Device, Queue, Extent2D};
use dacite::ext_debug_report::DebugReportCallbackExt;
use dacite::khr_surface::SurfaceKhr;
use winit::Window;

use self::setup::{Physical, QueueIndices};
use self::memory::Memory;
use self::swapchain_data::SwapchainData;
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
    // data_early: GpuDataEarly,
    // leading_data_late: GpuDataLate,
    // data_late: GpuDataLate,
    // command_buffer_early: CommandBuffer,
    // command_buffer_late: CommandBuffer,
    // fence_halfway: Fence,
    // fence_finished: Fence,
    // graphics_queue_early: Queue,
    // graphics_queue_late: Queue,

    swapchain_data: SwapchainData,
    memory: Memory,
    present_queue: Queue,
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

        let present_queue = device.get_queue(queue_indices.present_family,
                                             queue_indices.present_index);

        let memory = Memory::new(physical_device_memory_properties,
                                 &physical_device_properties);

        let swapchain_data = SwapchainData::create(
            &physical_device, &device, &surface,
            Extent2D { width: config.width, height: config.height },
            &queue_indices, config.vsync)?;

        Ok(Renderer {
            swapchain_data: swapchain_data,
            memory: memory,
            present_queue: present_queue,
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
        /*
        This interleaves the work of frame rendering such that two queues are
        operating in parallel, one working on the first phase (Early) of the
        leading frame and the second working on the second phase (Late) of
        the lagging frame.

        Note that we do not need multiple rust threads for this to occur.

        We transfer the ownership of the Shading buffer from the Early queue
        to the Late queue halfway in, and transfer it back at the end.

        We ensure that Phase-Early and Phase-Late commands do not utilize the
        same GPU writable data, except in read-only ways.

        Writable data utilized only during Phase-Early on the leading frame
        * Depth Buffer
        * Shading Buffer N (we have 1 per swapchain)

        Writable data utilized only during Phase-Late on the lagging frame
        * Shading Buffer N-1
        * Blur Buffer
        * Swapchain N-1

        We must upload data from the CPU to the GPU, and the GPU must not be
        reading the data while we are writing.  We uses fences to stop the GPU
        at the finished/halfway and halfway/finished points, then do the
        uploads, then submit command buffers again.

        We have to split the uploaded data into two sets, because the leading
        frame needs leading data, but the lagging frame ought to use lagging data
        to remain consistant with the prior work.  This means, for instance,
        two camera uniform sets.
        */

        // (data_early, leading_data_late) = UpdateData();
        // Upload (data_early)

        // submit CmdBufEarly half to QueueEarly, fence=fence_halfway
        // wait on fence_halfway

        loop {
            // bump swapchain index

            // data_late = leading_data_late;
            // (data_early, leading_data_late) = UpdateData();
            // Upload (data_early, data_late)

            // submit CmdBufEarly to QueueEarly, fence=draw_halfway
            // submit CmdBufLate to QueueLate, fence=draw_finished
            // wait on drawLate_finished
            // Present swapchain Late
            // wait on drawEarly_halfway
        }
    }
}
