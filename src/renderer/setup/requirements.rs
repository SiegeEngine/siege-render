/* Constants in this file should be updated as our situation changes,
 * as they are highly data and usage dependent */

use ash::vk::types::{Format, FormatFeatureFlags, FormatProperties, PhysicalDeviceFeatures,
                     FORMAT_FEATURE_COLOR_ATTACHMENT_BIT,
                     FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT,
                     FORMAT_FEATURE_SAMPLED_IMAGE_BIT, VK_KHR_SWAPCHAIN_EXTENSION_NAME};
use renderer::DeviceRequirements;

// This is the most common depth format supported on graphics hardware.
// (see http://vulkan.gpuinfo.org) and it is a good resolution, and it is
// floating-point (so reverse z-buffering works).
pub const DEPTH_FORMAT: Format = Format::D32Sfloat;
pub const DIFFUSE_FORMAT: Format = Format::A2b10g10r10UnormPack32;
pub const NORMALS_FORMAT: Format = Format::A2b10g10r10UnormPack32;
pub const MATERIAL_FORMAT: Format = Format::R8g8b8a8Unorm;
pub const SHADING_FORMAT: Format = Format::R16g16b16a16Sfloat;
pub const BLUR_FORMAT: Format = Format::R16g16b16a16Sfloat;

pub fn internal_requirements() -> DeviceRequirements {
    DeviceRequirements {
        extensions_required: vec![
            // ("VK_EXT_debug_report", 9), // This is always available and never output in enum
            // ("VK_KHR_surface", 25), // This is always available and never output in enum
            (VK_KHR_SWAPCHAIN_EXTENSION_NAME, 68),
        ],
        features_required: PhysicalDeviceFeatures {
            robust_buffer_access: cfg!(debug_assertions) as u32, // finds bugs; too expensive for live.
            ..Default::default()
        },
        formats_required: vec![
            // Depth buffer uses this
            (
                DEPTH_FORMAT,
                FormatProperties {
                    linear_tiling_features: FormatFeatureFlags::empty(),
                    optimal_tiling_features: FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT
                        | FORMAT_FEATURE_SAMPLED_IMAGE_BIT,
                    buffer_features: FormatFeatureFlags::empty(),
                },
            ),
            // Diffuse buffer uses this
            (
                DIFFUSE_FORMAT,
                FormatProperties {
                    linear_tiling_features: FormatFeatureFlags::empty(),
                    optimal_tiling_features: FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
                        | FORMAT_FEATURE_SAMPLED_IMAGE_BIT,
                    buffer_features: FormatFeatureFlags::empty(),
                },
            ),
            // Normals buffer uses this
            (
                NORMALS_FORMAT,
                FormatProperties {
                    linear_tiling_features: FormatFeatureFlags::empty(),
                    optimal_tiling_features: FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
                        | FORMAT_FEATURE_SAMPLED_IMAGE_BIT,
                    buffer_features: FormatFeatureFlags::empty(),
                },
            ),
            // Material buffer uses this
            (
                MATERIAL_FORMAT,
                FormatProperties {
                    linear_tiling_features: FormatFeatureFlags::empty(),
                    optimal_tiling_features: FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
                        | FORMAT_FEATURE_SAMPLED_IMAGE_BIT,
                    buffer_features: FormatFeatureFlags::empty(),
                },
            ),
            // Shading attachment uses this
            (
                SHADING_FORMAT,
                FormatProperties {
                    linear_tiling_features: FormatFeatureFlags::empty(),
                    optimal_tiling_features: FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
                        | FORMAT_FEATURE_SAMPLED_IMAGE_BIT,
                    buffer_features: FormatFeatureFlags::empty(),
                },
            ),
            // Blur attachment uses this
            (
                BLUR_FORMAT,
                FormatProperties {
                    linear_tiling_features: FormatFeatureFlags::empty(),
                    optimal_tiling_features: FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
                        | FORMAT_FEATURE_SAMPLED_IMAGE_BIT,
                    buffer_features: FormatFeatureFlags::empty(),
                },
            ),
        ],
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

// This is used to determine the size of the staging buffer
// pub const MAX_GPU_UPLOAD: u64 = ::renderer::memory::CHUNK_SIZE;
