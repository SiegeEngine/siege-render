use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::types::{ExtensionProperties, PhysicalDevice, PhysicalDeviceFeatures,
                     PhysicalDeviceMemoryProperties, PhysicalDeviceProperties, SurfaceKHR,
                     MEMORY_HEAP_DEVICE_LOCAL_BIT, MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
                     MEMORY_PROPERTY_HOST_CACHED_BIT, MEMORY_PROPERTY_HOST_COHERENT_BIT,
                     MEMORY_PROPERTY_HOST_VISIBLE_BIT, MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT};
use errors::*;
use renderer::queue_indices::QueueIndices;
use renderer::DeviceRequirements;
use std::ffi::CStr;

pub struct Physical {
    pub device: PhysicalDevice,
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub memory_properties: PhysicalDeviceMemoryProperties,
    pub queue_indices: QueueIndices,
    pub extensions: Vec<ExtensionProperties>,
}

pub fn find_suitable_device<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    surface: SurfaceKHR,
    requirements: &DeviceRequirements,
) -> Result<Physical> {
    // Add internal requirements to supplied requirements
    let requirements = requirements + &super::requirements::internal_requirements();

    // Get all suitable devices
    let suitable_devices: Vec<Physical> = instance
        .enumerate_physical_devices()?
        .into_iter()
        .filter_map(|physical_device| {
            match check_device_suitability(entry, instance, physical_device, surface, &requirements)
            {
                Ok(ds) => Some(ds),
                Err(e) => {
                    info!("{}", e);
                    None
                }
            }
        })
        .collect();

    // Choose the first suitable device, and log about it
    // FIXME: if we have more than one device that is suitable, let the user decide.
    match suitable_devices.into_iter().nth(0) {
        Some(ds) => {
            log_device_details(&ds);
            Ok(ds)
        }
        None => Err(ErrorKind::NoSuitableDevice.into()),
    }
}

fn check_device_suitability<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    requirements: &DeviceRequirements,
) -> Result<Physical> {
    let features = instance.get_physical_device_features(physical_device);

    // check features
    if !check_physical_device_features(&features, &requirements.features_required) {
        return Err(
            ErrorKind::DeviceNotSuitable("Device is missing a required feature".to_owned()).into(),
        );
    }

    // check formats required
    for (format, required_fmtprops) in &requirements.formats_required {
        let supported_fmtprops =
            instance.get_physical_device_format_properties(physical_device, *format);

        if !(required_fmtprops.linear_tiling_features & !supported_fmtprops.linear_tiling_features)
            .is_empty()
        {
            return Err(ErrorKind::DeviceNotSuitable(format!(
                "Device does not support format {:?} in linear tiling for {:?} \
                 (supports {:?})",
                format,
                required_fmtprops.linear_tiling_features,
                supported_fmtprops.linear_tiling_features
            )).into());
        }
        if !(required_fmtprops.optimal_tiling_features
            & !supported_fmtprops.optimal_tiling_features)
            .is_empty()
        {
            return Err(ErrorKind::DeviceNotSuitable(format!(
                "Device does not support format {:?} in optimal tiling for {:?} \
                 (supports {:?})",
                format,
                required_fmtprops.optimal_tiling_features,
                supported_fmtprops.optimal_tiling_features
            )).into());
        }
        if !(required_fmtprops.buffer_features & !supported_fmtprops.buffer_features).is_empty() {
            return Err(ErrorKind::DeviceNotSuitable(format!(
                "Device does not support format {:?} in buffers for {:?} \
                 (supports {:?})",
                format, required_fmtprops.buffer_features, supported_fmtprops.buffer_features
            )).into());
        }
    }

    // Check memory
    let memory_properties = instance.get_physical_device_memory_properties(physical_device);
    let mut device_memory: u64 = 0;
    for h in 0..memory_properties.memory_heap_count as usize {
        if memory_properties.memory_heaps[h]
            .flags
            .intersects(MEMORY_HEAP_DEVICE_LOCAL_BIT)
        {
            device_memory += memory_properties.memory_heaps[h].size;
        }
    }
    if device_memory < requirements.gpu_memory_required {
        return Err(ErrorKind::DeviceNotSuitable("Not enough memory".to_owned()).into());
    }

    // Check extensions
    let extension_properties = instance.enumerate_device_extension_properties(physical_device)?;
    let extensions_required = requirements.extensions_required.clone();
    'outer: for (rex, rexv) in &extensions_required {
        for aex in &extension_properties {
            // convert aex.extension_name to a &str
            let aexname = unsafe { CStr::from_ptr(aex.extension_name.as_ptr()).to_str() }?;
            if rex == &aexname {
                if aex.spec_version >= *rexv {
                    continue 'outer;
                } else {
                    return Err(ErrorKind::DeviceNotSuitable(format!(
                        "Extension {} version {} is too low ({} is required)",
                        rex, aex.spec_version, rexv
                    )).into());
                }
            }
        }
        return Err(ErrorKind::DeviceNotSuitable(format!("Extension {} is missing", rex)).into());
    }

    let properties = instance.get_physical_device_properties(physical_device);
    if properties.limits.max_push_constants_size < requirements.max_push_constants_size {
        return Err(ErrorKind::DeviceNotSuitable(format!(
            "Max push constants size is {} (required is {})",
            properties.limits.max_push_constants_size, requirements.max_push_constants_size
        )).into());
    }
    if properties.limits.max_color_attachments < 1 {
        return Err(ErrorKind::DeviceNotSuitable(format!(
            "Device does not support a single color attachment"
        )).into());
    }
    if properties.limits.max_framebuffer_layers < 1 {
        return Err(ErrorKind::DeviceNotSuitable(format!(
            "Device does not support a single framebuffer layer"
        )).into());
    }

    // QueueIndices
    let queue_indices = QueueIndices::new(entry, instance, physical_device, surface)?;

    Ok(Physical {
        device: physical_device,
        properties: properties,
        features: features,
        memory_properties: memory_properties,
        queue_indices: queue_indices,
        extensions: extension_properties,
    })
}

pub fn log_device_details(phys: &Physical) {
    let device_name = unsafe {
        CStr::from_ptr(phys.properties.device_name.as_ptr())
            .to_str()
            .unwrap()
    };
    info!("Using Graphics device: {}", device_name);

    info!(
        "Graphics Queues: Graphics({}[{}]), Present({}[{}]), Transfer({}[{}])",
        phys.queue_indices.graphics_family,
        phys.queue_indices.graphics_index,
        phys.queue_indices.present_family,
        phys.queue_indices.present_index,
        phys.queue_indices.transfer_family,
        phys.queue_indices.transfer_index
    );

    for h in 0..phys.memory_properties.memory_heap_count as usize {
        let heap = &phys.memory_properties.memory_heaps[h];
        let mut output = String::new();
        let flags = heap.flags;
        output.push_str(&*format!(
            "Heap type {:2} = {}MB - ",
            h,
            heap.size / 1048576
        ));
        if flags.intersects(MEMORY_HEAP_DEVICE_LOCAL_BIT) {
            output.push_str("Device Local");
        } else {
            output.push_str("Shared with Host");
        }
        info!("{}", output);
    }

    for t in 0..phys.memory_properties.memory_type_count as usize {
        let mtype = &phys.memory_properties.memory_types[t];
        let mut output = String::new();
        let flags = mtype.property_flags;
        output.push_str(&*format!(
            "Memory type {:2} on heap {:2} - ",
            t, mtype.heap_index
        ));
        if flags.intersects(MEMORY_PROPERTY_DEVICE_LOCAL_BIT) {
            output.push_str("DeviceLocal ");
        }
        if flags.intersects(MEMORY_PROPERTY_HOST_VISIBLE_BIT) {
            output.push_str("HostVisible ");
        }
        if flags.intersects(MEMORY_PROPERTY_HOST_COHERENT_BIT) {
            output.push_str("HostCoherent ");
        }
        if flags.intersects(MEMORY_PROPERTY_HOST_CACHED_BIT) {
            output.push_str("HostCached ");
        }
        if flags.intersects(MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT) {
            output.push_str("LazilyAllocated ");
        }
        info!("{}", output);
    }
}

fn check_physical_device_features(
    has: &PhysicalDeviceFeatures,
    req: &PhysicalDeviceFeatures,
) -> bool {
    (!req.robust_buffer_access | has.robust_buffer_access)
        & (!req.full_draw_index_uint32 | has.full_draw_index_uint32)
        & (!req.image_cube_array | has.image_cube_array)
        & (!req.independent_blend | has.independent_blend)
        & (!req.geometry_shader | has.geometry_shader)
        & (!req.tessellation_shader | has.tessellation_shader)
        & (!req.sample_rate_shading | has.sample_rate_shading)
        & (!req.dual_src_blend | has.dual_src_blend) & (!req.logic_op | has.logic_op)
        & (!req.multi_draw_indirect | has.multi_draw_indirect)
        & (!req.draw_indirect_first_instance | has.draw_indirect_first_instance)
        & (!req.depth_clamp | has.depth_clamp) & (!req.depth_bias_clamp | has.depth_bias_clamp)
        & (!req.fill_mode_non_solid | has.fill_mode_non_solid)
        & (!req.depth_bounds | has.depth_bounds) & (!req.wide_lines | has.wide_lines)
        & (!req.large_points | has.large_points) & (!req.alpha_to_one | has.alpha_to_one)
        & (!req.multi_viewport | has.multi_viewport)
        & (!req.sampler_anisotropy | has.sampler_anisotropy)
        & (!req.texture_compression_etc2 | has.texture_compression_etc2)
        & (!req.texture_compression_astc_ldr | has.texture_compression_astc_ldr)
        & (!req.texture_compression_bc | has.texture_compression_bc)
        & (!req.occlusion_query_precise | has.occlusion_query_precise)
        & (!req.pipeline_statistics_query | has.pipeline_statistics_query)
        & (!req.vertex_pipeline_stores_and_atomics | has.vertex_pipeline_stores_and_atomics)
        & (!req.fragment_stores_and_atomics | has.fragment_stores_and_atomics)
        & (!req.shader_tessellation_and_geometry_point_size
            | has.shader_tessellation_and_geometry_point_size)
        & (!req.shader_image_gather_extended | has.shader_image_gather_extended)
        & (!req.shader_storage_image_extended_formats | has.shader_storage_image_extended_formats)
        & (!req.shader_storage_image_multisample | has.shader_storage_image_multisample)
        & (!req.shader_storage_image_read_without_format
            | has.shader_storage_image_read_without_format)
        & (!req.shader_storage_image_write_without_format
            | has.shader_storage_image_write_without_format)
        & (!req.shader_uniform_buffer_array_dynamic_indexing
            | has.shader_uniform_buffer_array_dynamic_indexing)
        & (!req.shader_sampled_image_array_dynamic_indexing
            | has.shader_sampled_image_array_dynamic_indexing)
        & (!req.shader_storage_buffer_array_dynamic_indexing
            | has.shader_storage_buffer_array_dynamic_indexing)
        & (!req.shader_storage_image_array_dynamic_indexing
            | has.shader_storage_image_array_dynamic_indexing)
        & (!req.shader_clip_distance | has.shader_clip_distance)
        & (!req.shader_cull_distance | has.shader_cull_distance)
        & (!req.shader_float64 | has.shader_float64) & (!req.shader_int64 | has.shader_int64)
        & (!req.shader_int16 | has.shader_int16)
        & (!req.shader_resource_residency | has.shader_resource_residency)
        & (!req.shader_resource_min_lod | has.shader_resource_min_lod)
        & (!req.sparse_binding | has.sparse_binding)
        & (!req.sparse_residency_buffer | has.sparse_residency_buffer)
        & (!req.sparse_residency_image2d | has.sparse_residency_image2d)
        & (!req.sparse_residency_image3d | has.sparse_residency_image3d)
        & (!req.sparse_residency2samples | has.sparse_residency2samples)
        & (!req.sparse_residency4samples | has.sparse_residency4samples)
        & (!req.sparse_residency8samples | has.sparse_residency8samples)
        & (!req.sparse_residency16samples | has.sparse_residency16samples)
        & (!req.sparse_residency_aliased | has.sparse_residency_aliased)
        & (!req.variable_multisample_rate | has.variable_multisample_rate)
        & (!req.inherited_queries | has.inherited_queries) != 0
}
