/*
 * Constants in this file should be updated as our situation changes,
 * as they are highly data and usage dependent
 */

use dacite::core::{PhysicalDeviceFeatures, Format, FormatProperties};

// This is used to determine the size of the staging buffer
pub const MAX_GPU_UPLOAD: u64 = ::renderer::memory::CHUNK_SIZE;

// This is the most common depth format supported on graphics hardware.
// (see http://vulkan.gpuinfo.org) and it is a good resolution, and it is
// floating-point (so reverse z-buffering works).
pub const DEPTH_FORMAT: Format = Format::D32_SFloat;

pub const SHADING_IMAGE_FORMAT: Format = Format::R16G16B16A16_SFloat;

pub const BLUR_IMAGE_FORMAT: Format = Format::R16G16B16A16_SFloat;

pub const FEATURES_NEEDED: PhysicalDeviceFeatures = PhysicalDeviceFeatures {
    large_points: true,
    sampler_anisotropy: true, // FIXME - we want this, we dont need it.
    texture_compression_bc: true,
    //
    // the rest are false
    //
    robust_buffer_access: false,
    full_draw_index_uint32: false,
    image_cube_array: false,
    independent_blend: false,
    geometry_shader: false,
    tessellation_shader: false,
    sample_rate_shading: false,
    dual_src_blend: false,
    logic_op: false,
    multi_draw_indirect: false,
    draw_indirect_first_instance: false,
    depth_clamp: false,
    depth_bias_clamp: false,
    fill_mode_non_solid: false,
    depth_bounds: false,
    wide_lines: false,
    alpha_to_one: false,
    multi_viewport: false,
    texture_compression_etc2: false,
    texture_compression_astc_ldr: false,
    occlusion_query_precise: false,
    pipeline_statistics_query: false,
    vertex_pipeline_stores_and_atomics: false,
    fragment_stores_and_atomics: false,
    shader_tessellation_and_geometry_point_size: false,
    shader_image_gather_extended: false,
    shader_storage_image_extended_formats: false,
    shader_storage_image_multisample: false,
    shader_storage_image_read_without_format: false,
    shader_storage_image_write_without_format: false,
    shader_uniform_buffer_array_dynamic_indexing: false,
    shader_sampled_image_array_dynamic_indexing: false,
    shader_storage_buffer_array_dynamic_indexing: false,
    shader_storage_image_array_dynamic_indexing: false,
    shader_clip_distance: false,
    shader_cull_distance: false,
    shader_float64: false,
    shader_int64: false,
    shader_int16: false,
    shader_resource_residency: false,
    shader_resource_min_lod: false,
    sparse_binding: false,
    sparse_residency_buffer: false,
    sparse_residency_image_2d: false,
    sparse_residency_image_3d: false,
    sparse_residency_2_samples: false,
    sparse_residency_4_samples: false,
    sparse_residency_8_samples: false,
    sparse_residency_16_samples: false,
    sparse_residency_aliased: false,
    variable_multisample_rate: false,
    inherited_queries: false,
};

// FIXME: make a const fn once that is stable
pub fn get_formats_needed() ->  [(Format, FormatProperties); 5] {
    use dacite::core::FormatFeatureFlags;

    [
        // Most drawables use this format in the vertex buffer
        (Format::R32G32B32_SFloat, FormatProperties {
            linear_tiling_features: FormatFeatureFlags::empty(),
            optimal_tiling_features: FormatFeatureFlags::empty(),
            buffer_features: FormatFeatureFlags::VERTEX_BUFFER,
        }),
        // Depth buffer uses this
        (DEPTH_FORMAT, FormatProperties {
            linear_tiling_features: FormatFeatureFlags::empty(),
            optimal_tiling_features: FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
            buffer_features: FormatFeatureFlags::empty(),
        }),
        // Shading attachment uses this
        (SHADING_IMAGE_FORMAT, FormatProperties {
            linear_tiling_features: FormatFeatureFlags::empty(),
            optimal_tiling_features: FormatFeatureFlags::COLOR_ATTACHMENT,
            buffer_features: FormatFeatureFlags::empty(),
        }),
        // We will use this for crate_albedo.bc7.dds
        (Format::BC7_UNorm_Block, FormatProperties {
            linear_tiling_features: FormatFeatureFlags::empty(),
            optimal_tiling_features: FormatFeatureFlags::SAMPLED_IMAGE,
            buffer_features: FormatFeatureFlags::empty(),
        }),
        // We will use this for crate_normal.bc6h.dds
        (Format::BC6H_UFloat_Block, FormatProperties {
            linear_tiling_features: FormatFeatureFlags::empty(),
            optimal_tiling_features: FormatFeatureFlags::SAMPLED_IMAGE,
            buffer_features: FormatFeatureFlags::empty(),
        }),
        // Radeon R7 360 supports: BC1, BC2, BC3, BC4, BC5, BC6H, BC7
    ]
}

pub const PUSH_CONSTANTS_SIZE_REQUIRED: u32 = 0;
pub const COLOR_ATTACHMENT_COUNT_REQUIRED: u32 = 1;
pub const FRAMEBUFFER_LAYERS_REQUIRED: u32 = 1;
