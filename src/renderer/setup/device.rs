use ash::version::InstanceV1_0;
use ash::vk::types::{DeviceCreateFlags, DeviceCreateInfo, DeviceQueueCreateFlags,
                     DeviceQueueCreateInfo, PhysicalDeviceFeatures, StructureType};
use ash::Device;
use errors::*;
use renderer::requirements::DeviceRequirements;
use renderer::setup::physical::Physical;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;

pub fn create_device<I: InstanceV1_0>(
    instance: &I,
    physical: &Physical,
    requirements: &DeviceRequirements,
) -> Result<Device<I::Fp>> {
    let mut queues: HashMap<u32, u32> = HashMap::new();

    for &(fam, ind) in &[
        (
            physical.queue_indices.graphics_family,
            physical.queue_indices.graphics_index,
        ),
        (
            physical.queue_indices.present_family,
            physical.queue_indices.present_index,
        ),
        (
            physical.queue_indices.transfer_family,
            physical.queue_indices.transfer_index,
        ),
    ] {
        let mut entry = queues.entry(fam).or_insert(ind);
        if *entry < ind {
            *entry = ind;
        }
    }

    let device_queue_create_infos: Vec<DeviceQueueCreateInfo> = queues
        .iter()
        .map(|(family, maxqueue)| {
            let mut priorities: Vec<f32> = Vec::new();
            for _ in 0..maxqueue + 1 {
                priorities.push(1.0);
            }
            DeviceQueueCreateInfo {
                s_type: StructureType::DeviceQueueCreateInfo,
                p_next: ptr::null(),
                flags: DeviceQueueCreateFlags::empty(),
                queue_family_index: *family,
                queue_count: maxqueue + 1,
                p_queue_priorities: priorities.as_ptr(),
            }
        })
        .collect();

    let extension_names: Vec<*const i8> = physical
        .extensions
        .iter()
        .map(|ref e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()).as_ptr() })
        .collect();

    let device_create_info = DeviceCreateInfo {
        s_type: StructureType::DeviceCreateInfo,
        p_next: ptr::null(),
        flags: DeviceCreateFlags::empty(),
        queue_create_info_count: device_queue_create_infos.len() as u32,
        p_queue_create_infos: device_queue_create_infos.as_ptr(),
        enabled_layer_count: 0,
        pp_enabled_layer_names: ptr::null(),
        enabled_extension_count: physical.extensions.len() as u32,
        pp_enabled_extension_names: extension_names.as_ptr(),
        p_enabled_features: &requirements.features_required as *const PhysicalDeviceFeatures,
    };

    Ok(unsafe { instance.create_device(physical.device, &device_create_info, None) }?)
}
