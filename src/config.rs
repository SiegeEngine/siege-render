
use std::path::PathBuf;
use std::fmt;
use renderer::VulkanLogLevel;

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

#[derive(Clone, Deserialize)]
pub struct Config {
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
}

impl Default for Config {
    fn default() -> Config {
        Config {
            asset_path: default_asset_path(),
            vulkan_debug_output: default_vulkan_debug_output(),
            vulkan_log_level: default_vulkan_log_level(),
            vulkan_layers: default_vulkan_layers(),
            fps_cap: default_fps_cap(),
            reversed_depth_buffer: default_reversed_depth_buffer(),
            vsync: default_vsync(),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "    asset_path: {:?}", self.asset_path)?;
        writeln!(f, "    vulkan_debug_output: {:?}", self.vulkan_debug_output)?;
        writeln!(f, "    vulkan_log_level: {:?}", self.vulkan_log_level)?;
        writeln!(f, "    vulkan_log_layers:")?;
        for layer in &self.vulkan_layers {
            writeln!(f, "      {}", layer)?;
        }
        writeln!(f, "    reversed_depth_buffer: {:?}", self.reversed_depth_buffer)?;
        writeln!(f, "    FPS cap: {}", self.fps_cap)?;
        writeln!(f, "    vsync: {}", self.vsync)?;
        Ok(())
    }
}
