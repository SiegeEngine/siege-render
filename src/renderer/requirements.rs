use ash::vk::types::{Format, FormatProperties, PhysicalDeviceFeatures};
use std::ops::Add;

/// The set of requirements for a physical graphics device to be usable by the application.
pub struct DeviceRequirements {
    /// Extensions required and the version number they must be at or beyond
    pub extensions_required: Vec<(&'static str, u32)>,
    /// Physical device features that are required
    pub features_required: PhysicalDeviceFeatures,
    /// Formats that are required along with the required properties for each format.
    pub formats_required: Vec<(Format, FormatProperties)>,
    pub num_descriptor_sets_required: u32,
    pub num_uniform_buffers_required: u32,
    pub num_uniform_texel_buffers_required: u32,
    pub num_dynamic_uniform_buffers_required: u32,
    pub num_samplers_required: u32,
    pub num_sampled_images_required: u32,
    pub num_combined_image_samplers_required: u32,
    /// The amount of GPU memory required
    pub gpu_memory_required: u64,
    /// Bytes required for push constants. Vulkan guarantees 128 bytes for push constants, so you
    /// only need to set this if you use more.
    pub max_push_constants_size: u32,
}

impl Default for DeviceRequirements {
    fn default() -> DeviceRequirements {
        DeviceRequirements {
            extensions_required: vec![],
            features_required: Default::default(),
            formats_required: vec![],
            num_descriptor_sets_required: 0,
            num_uniform_buffers_required: 0,
            num_uniform_texel_buffers_required: 0,
            num_dynamic_uniform_buffers_required: 0,
            num_samplers_required: 0,
            num_sampled_images_required: 0,
            num_combined_image_samplers_required: 0,
            gpu_memory_required: 0,
            max_push_constants_size: 0,
        }
    }
}

impl<'a> Add for &'a DeviceRequirements {
    type Output = DeviceRequirements;

    fn add(self, other: &'a DeviceRequirements) -> DeviceRequirements {
        DeviceRequirements {
            extensions_required: {
                let mut er = self.extensions_required.clone();
                'outer: for oer in &other.extensions_required {
                    for i in 0..er.len() {
                        if oer.0 == er[i].0 {
                            er[i].1 = er[i].1.max(oer.1);
                            continue 'outer;
                        }
                    }
                    er.push(oer.clone());
                }
                er
            },
            features_required: add_physical_device_features(
                &self.features_required,
                &other.features_required,
            ),
            formats_required: {
                let mut v = self.formats_required.clone();
                v.extend(other.formats_required.clone());
                v
            },
            num_descriptor_sets_required: self.num_descriptor_sets_required
                + other.num_descriptor_sets_required,
            num_uniform_buffers_required: self.num_uniform_buffers_required
                + other.num_uniform_buffers_required,
            num_uniform_texel_buffers_required: self.num_uniform_texel_buffers_required
                + other.num_uniform_texel_buffers_required,
            num_dynamic_uniform_buffers_required: self.num_dynamic_uniform_buffers_required
                + other.num_dynamic_uniform_buffers_required,
            num_samplers_required: self.num_samplers_required + other.num_samplers_required,
            num_sampled_images_required: self.num_sampled_images_required
                + other.num_sampled_images_required,
            num_combined_image_samplers_required: self.num_combined_image_samplers_required
                + other.num_combined_image_samplers_required,
            gpu_memory_required: self.gpu_memory_required + other.gpu_memory_required,
            max_push_constants_size: self.max_push_constants_size
                .max(other.max_push_constants_size),
        }
    }
}

fn add_physical_device_features(
    a: &PhysicalDeviceFeatures,
    b: &PhysicalDeviceFeatures,
) -> PhysicalDeviceFeatures {
    PhysicalDeviceFeatures {
        robust_buffer_access: a.robust_buffer_access | b.robust_buffer_access,
        full_draw_index_uint32: a.full_draw_index_uint32 | b.full_draw_index_uint32,
        image_cube_array: a.image_cube_array | b.image_cube_array,
        independent_blend: a.independent_blend | b.independent_blend,
        geometry_shader: a.geometry_shader | b.geometry_shader,
        tessellation_shader: a.tessellation_shader | b.tessellation_shader,
        sample_rate_shading: a.sample_rate_shading | b.sample_rate_shading,
        dual_src_blend: a.dual_src_blend | b.dual_src_blend,
        logic_op: a.logic_op | b.logic_op,
        multi_draw_indirect: a.multi_draw_indirect | b.multi_draw_indirect,
        draw_indirect_first_instance: a.draw_indirect_first_instance
            | b.draw_indirect_first_instance,
        depth_clamp: a.depth_clamp | b.depth_clamp,
        depth_bias_clamp: a.depth_bias_clamp | b.depth_bias_clamp,
        fill_mode_non_solid: a.fill_mode_non_solid | b.fill_mode_non_solid,
        depth_bounds: a.depth_bounds | b.depth_bounds,
        wide_lines: a.wide_lines | b.wide_lines,
        large_points: a.large_points | b.large_points,
        alpha_to_one: a.alpha_to_one | b.alpha_to_one,
        multi_viewport: a.multi_viewport | b.multi_viewport,
        sampler_anisotropy: a.sampler_anisotropy | b.sampler_anisotropy,
        texture_compression_etc2: a.texture_compression_etc2 | b.texture_compression_etc2,
        texture_compression_astc_ldr: a.texture_compression_astc_ldr
            | b.texture_compression_astc_ldr,
        texture_compression_bc: a.texture_compression_bc | b.texture_compression_bc,
        occlusion_query_precise: a.occlusion_query_precise | b.occlusion_query_precise,
        pipeline_statistics_query: a.pipeline_statistics_query | b.pipeline_statistics_query,
        vertex_pipeline_stores_and_atomics: a.vertex_pipeline_stores_and_atomics
            | b.vertex_pipeline_stores_and_atomics,
        fragment_stores_and_atomics: a.fragment_stores_and_atomics | b.fragment_stores_and_atomics,
        shader_tessellation_and_geometry_point_size: a.shader_tessellation_and_geometry_point_size
            | b.shader_tessellation_and_geometry_point_size,
        shader_image_gather_extended: a.shader_image_gather_extended
            | b.shader_image_gather_extended,
        shader_storage_image_extended_formats: a.shader_storage_image_extended_formats
            | b.shader_storage_image_extended_formats,
        shader_storage_image_multisample: a.shader_storage_image_multisample
            | b.shader_storage_image_multisample,
        shader_storage_image_read_without_format: a.shader_storage_image_read_without_format
            | b.shader_storage_image_read_without_format,
        shader_storage_image_write_without_format: a.shader_storage_image_write_without_format
            | b.shader_storage_image_write_without_format,
        shader_uniform_buffer_array_dynamic_indexing: a.shader_uniform_buffer_array_dynamic_indexing
            | b.shader_uniform_buffer_array_dynamic_indexing,
        shader_sampled_image_array_dynamic_indexing: a.shader_sampled_image_array_dynamic_indexing
            | b.shader_sampled_image_array_dynamic_indexing,
        shader_storage_buffer_array_dynamic_indexing: a.shader_storage_buffer_array_dynamic_indexing
            | b.shader_storage_buffer_array_dynamic_indexing,
        shader_storage_image_array_dynamic_indexing: a.shader_storage_image_array_dynamic_indexing
            | b.shader_storage_image_array_dynamic_indexing,
        shader_clip_distance: a.shader_clip_distance | b.shader_clip_distance,
        shader_cull_distance: a.shader_cull_distance | b.shader_cull_distance,
        shader_float64: a.shader_float64 | b.shader_float64,
        shader_int64: a.shader_int64 | b.shader_int64,
        shader_int16: a.shader_int16 | b.shader_int16,
        shader_resource_residency: a.shader_resource_residency | b.shader_resource_residency,
        shader_resource_min_lod: a.shader_resource_min_lod | b.shader_resource_min_lod,
        sparse_binding: a.sparse_binding | b.sparse_binding,
        sparse_residency_buffer: a.sparse_residency_buffer | b.sparse_residency_buffer,
        sparse_residency_image2d: a.sparse_residency_image2d | b.sparse_residency_image2d,
        sparse_residency_image3d: a.sparse_residency_image3d | b.sparse_residency_image3d,
        sparse_residency2samples: a.sparse_residency2samples | b.sparse_residency2samples,
        sparse_residency4samples: a.sparse_residency4samples | b.sparse_residency4samples,
        sparse_residency8samples: a.sparse_residency8samples | b.sparse_residency8samples,
        sparse_residency16samples: a.sparse_residency16samples | b.sparse_residency16samples,
        sparse_residency_aliased: a.sparse_residency_aliased | b.sparse_residency_aliased,
        variable_multisample_rate: a.variable_multisample_rate | b.variable_multisample_rate,
        inherited_queries: a.inherited_queries | b.inherited_queries,
    }
}
