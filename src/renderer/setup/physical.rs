
use dacite::core::{Instance, PhysicalDevice, PhysicalDeviceProperties,
                   PhysicalDeviceFeatures, PhysicalDeviceLimits,
                   PhysicalDeviceMemoryProperties, DeviceExtensions,
                   DeviceExtensionsProperties, Format, FormatProperties};
use dacite::khr_surface::SurfaceKhr;

use errors::*;
use super::requirements::*;
use super::QueueIndices;
use config::Config;

pub struct Physical {
    pub physical_device: PhysicalDevice,
    pub physical_device_properties: PhysicalDeviceProperties,
    pub physical_device_features: PhysicalDeviceFeatures,
    pub physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    pub queue_indices: QueueIndices,
    pub device_extensions: DeviceExtensions,
}

pub fn find_suitable_device(
    config: &Config,
    instance: &Instance,
    surface: &SurfaceKhr)
    -> Result<Physical>
{
    let formats_needed = get_formats_needed();

    let devices: Vec<Physical> = instance.enumerate_physical_devices()?.into_iter()
        .filter_map(|physical_device| {
            match check_device_suitability(
                config,
                physical_device,
                surface,
                FEATURES_NEEDED,
                &formats_needed)
            {
                Ok(ds) => Some(ds),
                Err(e) => {
                    info!("{}", e);
                    None
                }
            }
        })
        .collect();

    // FIXME: let the user choose the device to use
    // if devices.len() > 1 {

    match devices.into_iter().nth(0) {
        Some(ds) => {
            log_device_details(&ds);
            Ok(ds)
        },
        None => Err(ErrorKind::NoSuitableDevice.into())
    }
}


fn check_device_suitability(
    config: &Config,
    physical_device: PhysicalDevice,
    surface: &SurfaceKhr,
    features_needed: PhysicalDeviceFeatures,
    formats_needed: &[(Format, FormatProperties)])
    -> Result<Physical>
{
    let physical_device_properties = physical_device.get_properties();

    let physical_device_features = check_physical_device_features(
        &physical_device, features_needed)?;

    let physical_device_memory_properties = check_physical_device_memory_properties(
        config,
        &physical_device)?;

    let queue_indices = QueueIndices::new(&physical_device, surface)?;

    let device_extensions = check_device_extensions(&physical_device)?;

    check_limits(&physical_device_properties.limits)?;

    for &(format, required) in formats_needed {
        let supported = physical_device.get_format_properties(format);
        if ! supported.linear_tiling_features.contains(required.linear_tiling_features) {
            return Err(ErrorKind::DeviceNotSuitable(
                format!("Device does not support format {:?} in linear tiling for {:?} \
                         (supports {:?})", format,
                        required.linear_tiling_features, supported.linear_tiling_features))
                       .into());
        }
        if ! supported.optimal_tiling_features.contains(required.optimal_tiling_features) {
            return Err(ErrorKind::DeviceNotSuitable(
                format!("Device does not support format {:?} in optimal tiling for {:?} \
                         (supports {:?})", format,
                        required.optimal_tiling_features, supported.optimal_tiling_features))
                       .into());
        }
        if ! supported.buffer_features.contains(required.buffer_features) {
            return Err(ErrorKind::DeviceNotSuitable(
                format!("Device does not support format {:?} in buffers for {:?} \
                         (supports {:?})", format,
                        required.buffer_features, supported.buffer_features))
                       .into());
        }
    }

    Ok(Physical {
        physical_device: physical_device,
        physical_device_properties: physical_device_properties,
        physical_device_features: physical_device_features,
        physical_device_memory_properties: physical_device_memory_properties,
        queue_indices: queue_indices,
        device_extensions: device_extensions,
    })
}

fn check_physical_device_features(
    physical_device: &PhysicalDevice,
    mut features_needed: PhysicalDeviceFeatures)
    -> Result<PhysicalDeviceFeatures>
{
    // See https://www.khronos.org/registry/vulkan/specs/1.0/man/html/VkPhysicalDeviceFeatures.html
    let features_available = physical_device.get_features();

    features_needed.difference(&features_available);// subtract out available features
    if !features_needed.is_empty() {
        // Some feature that we need is not available. Unfortunately it's hard to tell
        // which without a very long set of if/then statements.
        Err(ErrorKind::DeviceNotSuitable(
            "Device is missing a required feature".to_owned()).into())
    } else {
        Ok(features_available)
    }
}

fn check_physical_device_memory_properties(
    config: &Config,
    physical_device: &PhysicalDevice)
    -> Result<PhysicalDeviceMemoryProperties>
{
    use dacite::core::MemoryHeapFlags;

    let memory_properties = physical_device.get_memory_properties();

    // Verify we have enough DEVICE_LOCAL memory:
    let device_memory = memory_properties.memory_heaps
        .iter()
        .filter(|&x| x.flags.contains(MemoryHeapFlags::DEVICE_LOCAL))
        .fold(0, |acc, &x| acc + x.size);
    if device_memory < config.gpu_memory_required {
        return Err(ErrorKind::DeviceNotSuitable("Not enough memory".to_owned()).into());
    }

    Ok(memory_properties)
}

fn check_device_extensions(physical_device: &PhysicalDevice) -> Result<DeviceExtensions>
{
    let available_extensions = physical_device.get_device_extension_properties(None)?;
    let mut required_extensions = DeviceExtensionsProperties::new();
    required_extensions.add_khr_swapchain(67); // spec version 67

    let missing_extensions = required_extensions.difference(&available_extensions);
    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    }
    else {
        let mut s = String::new();
        for (name, spec_version) in missing_extensions.properties() {
            s.push_str(&*format!("Extension {} (revision {}) missing", name, spec_version));
        }
        Err(ErrorKind::MissingExtensions(s).into())
    }
}

fn check_limits(limits: &PhysicalDeviceLimits) -> Result<()>
{
    if limits.max_push_constants_size < PUSH_CONSTANTS_SIZE_REQUIRED {
        return Err(ErrorKind::DeviceNotSuitable(
            "Not enough space for push constants".to_owned()).into());
    }
    if limits.max_color_attachments < COLOR_ATTACHMENT_COUNT_REQUIRED {
        return Err(ErrorKind::DeviceNotSuitable(
            "Not enough color attachments available".to_owned()).into());
    }
    // We dont need to check max_width/max_height; We cant alter the window maximums
    // post-creation; we get current_extent from the surface extension; we presume
    // it stays within bounds of max_framebuffer_width/height.
    if limits.max_framebuffer_layers < FRAMEBUFFER_LAYERS_REQUIRED {
        return Err(ErrorKind::DeviceNotSuitable(
            "Not enough framebuffer layers".to_owned()).into());
    }

    Ok(())
}

pub fn log_device_details(phys: &Physical)
{
    use dacite::core::{MemoryPropertyFlags, MemoryHeapFlags};

    info!("Using Graphics device: {}", phys.physical_device_properties.device_name);

    info!("Graphics Queues: Graphics({}[{}]), Present({}[{}]), Transfer({}[{}])",
          phys.queue_indices.graphics_family, phys.queue_indices.graphics_index,
          phys.queue_indices.present_family, phys.queue_indices.present_index,
          phys.queue_indices.transfer_family, phys.queue_indices.transfer_index);

    // display some memory properties
    for (i,heap) in phys.physical_device_memory_properties.memory_heaps.iter().enumerate() {
        let mut output = String::new();
        let flags = heap.flags;
        output.push_str(&*format!("Heap type {:2} = {}MB - ", i, heap.size / 1048576));
        if flags.contains(MemoryHeapFlags::DEVICE_LOCAL) {
            output.push_str("Device Local");
        }
        else {
            output.push_str("Shared with Host");
        }
        info!("{}", output);
    }
    for (i,mt) in phys.physical_device_memory_properties.memory_types.iter().enumerate() {
        let mut output = String::new();
        let flags = mt.property_flags;
        output.push_str(&*format!("Memory type {:2} on heap {:2} - ", i, mt.heap_index));
        if flags.contains(MemoryPropertyFlags::DEVICE_LOCAL) {
            output.push_str("DeviceLocal ");
        }
        if flags.contains(MemoryPropertyFlags::HOST_VISIBLE) {
            output.push_str("HostVisible ");
        }
        if flags.contains(MemoryPropertyFlags::HOST_COHERENT) {
            output.push_str("HostCoherent ");
        }
        if flags.contains(MemoryPropertyFlags::HOST_CACHED) {
            output.push_str("HostCached ");
        }
        if flags.contains(MemoryPropertyFlags::LAZILY_ALLOCATED) {
            output.push_str("LazilyAllocated ");
        }
        info!("{}", output);
    }
}
