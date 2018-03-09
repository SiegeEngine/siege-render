
pub mod requirements;

mod queue_indices;
pub use self::queue_indices::QueueIndices;

mod physical;
pub use self::physical::{Physical, find_suitable_device};

use std::sync::Arc;
use std::collections::HashMap;
use dacite::core::{InstanceExtensions, Instance, PhysicalDevice, DeviceExtensions,
                   Device, Semaphore, Fence, DescriptorPool};
use dacite::ext_debug_report::{DebugReportFlagsExt, DebugReportObjectTypeExt,
                               DebugReportCallbackExt, DebugReportCallbacksExt};
use dacite::khr_surface::SurfaceKhr;
use dacite_winit::WindowExt;
use winit::Window;

use self::requirements::FEATURES_NEEDED;
use config::Config;
use errors::*;
use renderer::VulkanLogLevel;

pub fn setup_instance(config: &Config, window: &Window) -> Result<Instance>
{
    let create_info = {
        use dacite::core::{InstanceCreateFlags, InstanceCreateInfo,
                           ApplicationInfo, Version};

        let mut extensions = compute_instance_extensions(window)?;

        if config.vulkan_debug_output {
            extensions.add_ext_debug_report();
        }

        InstanceCreateInfo {
            flags: InstanceCreateFlags::empty(),
            application_info: Some(ApplicationInfo {
                application_name: Some("Eye of Baal".to_owned()),
                application_version: Version {
                    major: config.major_version,
                    minor: config.minor_version,
                    patch: config.patch_version,
                }.as_api_version(),
                engine_name: Some("Siege Engine".to_owned()),
                engine_version: Version {
                    major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                    minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                    patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap()
                }.as_api_version(),
                api_version: Some(Version {
                    major: 1,
                    minor: 0,
                    patch: 3,
                }),
                chain: None,
            }),
            enabled_layers: config.vulkan_layers.clone(),
            enabled_extensions: extensions,
            chain: None,
        }
    };

    Ok(Instance::create(&create_info, None)?)
}


fn compute_instance_extensions(window: &Window) -> Result<InstanceExtensions>
{

    let available_extensions = Instance::get_instance_extension_properties(None)?;

    let required_extensions = window.get_required_extensions()?;

    let missing_extensions = required_extensions.difference(&available_extensions);

    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    } else {
        let mut s = String::new();
        for (name, spec_version) in missing_extensions.properties() {
            s.push_str(&*format!("Extension {} (revision {})", name, spec_version));
        }
        Err(ErrorKind::MissingExtensions(s).into())
    }
}

pub fn setup_debug_callback(config: &Config, instance: &Instance)
                            -> Result<Option<DebugReportCallbackExt>>
{
    if config.vulkan_debug_output {
        use dacite::ext_debug_report::{
            DebugReportCallbackCreateInfoExt, DebugReportFlagsExt};

        let flags = {
            let mut flags = DebugReportFlagsExt::ERROR;
            if config.vulkan_log_level >= VulkanLogLevel::Warning {
                flags |= DebugReportFlagsExt::WARNING;
            }
            if config.vulkan_log_level >= VulkanLogLevel::PerformanceWarning {
                flags |= DebugReportFlagsExt::PERFORMANCE_WARNING;
            }
            if config.vulkan_log_level >= VulkanLogLevel::Information {
                flags |= DebugReportFlagsExt::INFORMATION;
            }
            if config.vulkan_log_level >= VulkanLogLevel::Debug {
                flags |= DebugReportFlagsExt::DEBUG;
            }
            flags
        };

        let create_info = DebugReportCallbackCreateInfoExt {
            flags: flags,
            callback: Arc::new(DebugCallback),
            chain: None,
        };

        let debug_callback = instance.create_debug_report_callback_ext(&create_info, None)?;
        Ok(Some(debug_callback))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
struct DebugCallback;

impl DebugReportCallbacksExt for DebugCallback {
    fn callback(
        &self,
        flags: DebugReportFlagsExt,
        _object_type: DebugReportObjectTypeExt,
        _object: u64,
        _location: usize,
        _message_code: i32,
        _layer_prefix: Option<&str>,
        message: Option<&str>) -> bool
    {
        if let Some(m) = message {
            if flags.intersects(DebugReportFlagsExt::ERROR) {
                error!("\r\n  vk: {}", m);
            }
            else if flags.intersects(DebugReportFlagsExt::WARNING) {
                warn!("\r\n  vk: {}", m);
            }
            else if flags.intersects(DebugReportFlagsExt::PERFORMANCE_WARNING) {
                warn!("\r\n  vk: {}", m);
            }
            else if flags.intersects(DebugReportFlagsExt::INFORMATION) {
                info!("\r\n  vk: {}", m);
            }
            else if flags.intersects(DebugReportFlagsExt::DEBUG) {
                debug!("\r\n  vk: {}", m);
            }
        }

        // We should return true here ONLY IF this was a validation ERROR (not warning
        // or info).
        //
        // We want to fail on warnings too
        flags.intersects(DebugReportFlagsExt::ERROR | DebugReportFlagsExt::WARNING)
    }
}

pub fn setup_surface(window: &Window, instance: &Instance) -> Result<SurfaceKhr>
{
    use dacite_winit::SurfaceCreateFlags;
    Ok(window.create_surface(
        &instance,
        SurfaceCreateFlags::empty(),
        None)?)
}

pub fn create_device(physical_device: &PhysicalDevice,
                     device_extensions: DeviceExtensions,
                     queue_indices: &QueueIndices)
                     -> Result<Device>
{
    use dacite::core::{DeviceQueueCreateInfo, DeviceQueueCreateFlags,
                       DeviceCreateInfo, DeviceCreateFlags};

    let mut queues: HashMap<u32, u32> = HashMap::new();

    for &(fam,ind) in &[(queue_indices.graphics_family, queue_indices.graphics_index),
                        (queue_indices.present_family, queue_indices.present_index),
                        (queue_indices.transfer_family, queue_indices.transfer_index)]
    {
        let mut entry = queues.entry(fam).or_insert(ind);
        if *entry < ind {  *entry = ind; }
    }

    let device_queue_create_infos = queues.iter().map(|(family,maxqueue)| {
        let mut priorities: Vec<f32> = Vec::new();
        for _ in 0..maxqueue+1 {
            priorities.push(1.0);
        }
        DeviceQueueCreateInfo {
            flags: DeviceQueueCreateFlags::empty(),
            queue_family_index: *family,
            queue_priorities: priorities,
            chain: None,
        }
    }).collect();

    let device_create_info = DeviceCreateInfo {
        flags: DeviceCreateFlags::empty(),
        queue_create_infos: device_queue_create_infos,
        enabled_layers: vec![],
        enabled_extensions: device_extensions,
        enabled_features: Some(FEATURES_NEEDED),
        chain: None,
    };

    Ok(physical_device.create_device(&device_create_info, None)?)
}

pub fn get_descriptor_pool(device: &Device, config: &Config) -> Result<DescriptorPool>
{
    use dacite::core::{DescriptorPoolCreateInfo, DescriptorPoolSize,
                               DescriptorType};

    let create_info = DescriptorPoolCreateInfo {
        flags: Default::default(),
        max_sets: config.max_descriptor_sets,
        pool_sizes: vec![
            DescriptorPoolSize {
                descriptor_type: DescriptorType::UniformBuffer,
                descriptor_count: config.max_uniform_buffers,
            },
            DescriptorPoolSize {
                descriptor_type: DescriptorType::UniformTexelBuffer,
                descriptor_count: config.max_uniform_texel_buffers,
            },
            DescriptorPoolSize {
                descriptor_type: DescriptorType::UniformBufferDynamic,
                descriptor_count: config.max_dynamic_uniform_buffers,
            },
            DescriptorPoolSize {
                descriptor_type: DescriptorType::Sampler,
                descriptor_count: config.max_samplers,
            },
            DescriptorPoolSize {
                descriptor_type: DescriptorType::SampledImage,
                descriptor_count: config.max_sampled_images,
            },
            DescriptorPoolSize {
                descriptor_type: DescriptorType::CombinedImageSampler,
                descriptor_count: config.max_combined_image_samplers,
            },
        ],
        chain: None,
    };

    Ok(device.create_descriptor_pool(&create_info, None)?)

}

pub fn get_semaphores(device: &Device) -> Result<(Semaphore, Semaphore)>
{
    use dacite::core::{SemaphoreCreateInfo, SemaphoreCreateFlags};

    let create_info = SemaphoreCreateInfo {
        flags: SemaphoreCreateFlags::empty(),
        chain: None,
    };

    let image_acquired = device.create_semaphore(&create_info, None)?;
    let image_rendered = device.create_semaphore(&create_info, None)?;

    Ok((image_acquired, image_rendered))
}


pub fn get_graphics_fence(device: &Device, signalled: bool) -> Result<Fence>
{
    use dacite::core::{FenceCreateInfo, FenceCreateFlags};
    let create_info = FenceCreateInfo {
        flags: if signalled {
            FenceCreateFlags::SIGNALED
        } else {
            FenceCreateFlags::empty()
        },
        chain: None
    };
    Ok(device.create_fence(&create_info, None)?)
}
