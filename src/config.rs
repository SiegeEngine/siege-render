
use std::path::PathBuf;
use std::fmt;
use renderer::VulkanLogLevel;

#[inline] fn default_major_version() -> u32 { 0 }
#[inline] fn default_minor_version() -> u32 { 1 }
#[inline] fn default_patch_version() -> u32 { 0 }
#[inline] fn default_asset_path() -> PathBuf { PathBuf::from("assets") }
#[inline] fn default_vulkan_debug_output() -> bool { cfg!(debug_assertions) }
#[inline] fn default_vulkan_log_level() -> VulkanLogLevel {
    if cfg!(debug_assertions) { VulkanLogLevel::Debug }
    else { VulkanLogLevel::PerformanceWarning }
}
#[inline] fn default_vulkan_layers() -> Vec<String> {
    vec![]
}
#[inline] fn default_fps_cap() -> u32 { 120 }
#[inline] fn default_reversed_depth_buffer() -> bool {
    true
}
#[inline] fn default_vsync() -> bool { true }
#[inline] fn default_width() -> u32 { 800 }
#[inline] fn default_height() -> u32 { 600 }
#[inline] fn default_gpu_memory_required() -> u64 { 9216000 }
#[inline] fn default_max_descriptor_sets() -> u32 { 16 }
#[inline] fn default_max_uniform_buffers() -> u32 { 2 }
#[inline] fn default_max_dynamic_uniform_buffers() -> u32 { 2 }
#[inline] fn default_max_samplers() -> u32 { 2 }
#[inline] fn default_max_sampled_images() -> u32 { 2 }
#[inline] fn default_max_combined_image_samplers() -> u32 { 10 }

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_major_version")]
    pub major_version: u32,
    #[serde(default = "default_minor_version")]
    pub minor_version: u32,
    #[serde(default = "default_patch_version")]
    pub patch_version: u32,
    #[serde(default = "default_asset_path")]
    pub asset_path: PathBuf,
    #[serde(default = "default_vulkan_debug_output")]
    pub vulkan_debug_output: bool,
    #[serde(default = "default_vulkan_log_level")]
    pub vulkan_log_level: VulkanLogLevel,
    #[serde(default = "default_vulkan_layers")]
    pub vulkan_layers: Vec<String>,
    #[serde(default = "default_fps_cap")]
    pub fps_cap: u32,
    #[serde(default = "default_reversed_depth_buffer")]
    pub reversed_depth_buffer: bool,
    #[serde(default = "default_vsync")]
    pub vsync: bool,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_gpu_memory_required")]
    pub gpu_memory_required: u64,
    #[serde(default = "default_max_descriptor_sets")]
    pub max_descriptor_sets: u32,
    #[serde(default = "default_max_uniform_buffers")]
    pub max_uniform_buffers: u32,
    #[serde(default = "default_max_dynamic_uniform_buffers")]
    pub max_dynamic_uniform_buffers: u32,
    #[serde(default = "default_max_samplers")]
    pub max_samplers: u32,
    #[serde(default = "default_max_sampled_images")]
    pub max_sampled_images: u32,
    #[serde(default = "default_max_combined_image_samplers")]
    pub max_combined_image_samplers: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            major_version: default_major_version(),
            minor_version: default_minor_version(),
            patch_version: default_patch_version(),
            asset_path: default_asset_path(),
            vulkan_debug_output: default_vulkan_debug_output(),
            vulkan_log_level: default_vulkan_log_level(),
            vulkan_layers: default_vulkan_layers(),
            fps_cap: default_fps_cap(),
            reversed_depth_buffer: default_reversed_depth_buffer(),
            vsync: default_vsync(),
            width: default_width(),
            height: default_height(),
            gpu_memory_required: default_gpu_memory_required(),
            max_descriptor_sets: default_max_descriptor_sets(),
            max_uniform_buffers: default_max_uniform_buffers(),
            max_dynamic_uniform_buffers: default_max_dynamic_uniform_buffers(),
            max_samplers: default_max_samplers(),
            max_sampled_images: default_max_sampled_images(),
            max_combined_image_samplers: default_max_combined_image_samplers(),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "    App version: {}.{}.{}",
                 self.major_version, self.minor_version, self.patch_version)?;
        writeln!(f, "    Renderer version: {}.{}.{}",
                 env!("CARGO_PKG_VERSION_MAJOR"),
                 env!("CARGO_PKG_VERSION_MINOR"),
                 env!("CARGO_PKG_VERSION_PATCH"))?;
        writeln!(f, "    Asset path: {:?}", self.asset_path)?;
        writeln!(f, "    Vulkan debug output: {:?}", self.vulkan_debug_output)?;
        writeln!(f, "    Vulkan log level: {:?}", self.vulkan_log_level)?;
        writeln!(f, "    Vulkan log layers:")?;
        for layer in &self.vulkan_layers {
            writeln!(f, "      {}", layer)?;
        }
        writeln!(f, "    Reversed depth buffer: {:?}", self.reversed_depth_buffer)?;
        writeln!(f, "    FPS cap: {}", self.fps_cap)?;
        writeln!(f, "    vsync: {}", self.vsync)?;
        writeln!(f, "    width: {}", self.width)?;
        writeln!(f, "    height: {}", self.height)?;
        writeln!(f, "    have a least: {} memory", self.gpu_memory_required)?;
        writeln!(f, "    Allocated desc sets: {}", self.max_descriptor_sets)?;
        writeln!(f, "    Allocated desc for uniform buffers: {}", self.max_uniform_buffers)?;
        writeln!(f, "    Allocated desc for dynamic uniform buffers: {}", self.max_dynamic_uniform_buffers)?;
        writeln!(f, "    Allocated desc for samplers: {}", self.max_samplers)?;
        writeln!(f, "    Allocated desc for sampled images: {}", self.max_sampled_images)?;
        writeln!(f, "    Allocated desc for combined image samplers: {}", self.max_combined_image_samplers)?;

        Ok(())
    }
}
