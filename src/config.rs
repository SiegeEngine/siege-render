use renderer::{Tonemapper, VulkanLogLevel};
use std::fmt;
use std::path::PathBuf;

#[inline]
fn default_app_name() -> String {
    "Unspecified".to_owned()
}
#[inline]
fn default_major_version() -> u32 {
    0
}
#[inline]
fn default_minor_version() -> u32 {
    1
}
#[inline]
fn default_patch_version() -> u32 {
    0
}
#[inline]
fn default_asset_path() -> PathBuf {
    PathBuf::from("assets")
}
#[inline]
fn default_vulkan_debug_output() -> bool {
    cfg!(debug_assertions)
}
#[inline]
fn default_vulkan_log_level() -> VulkanLogLevel {
    if cfg!(debug_assertions) {
        VulkanLogLevel::Debug
    } else {
        VulkanLogLevel::PerformanceWarning
    }
}
#[inline]
fn default_vulkan_layers() -> Vec<String> {
    vec![]
}
#[inline]
fn default_fps_cap() -> u32 {
    120
}
#[inline]
fn default_reversed_depth_buffer() -> bool {
    true
}
#[inline]
fn default_width() -> u32 {
    800
}
#[inline]
fn default_height() -> u32 {
    600
}
#[inline]
fn default_display_luminance() -> u32 {
    80
}
#[inline]
fn default_timing_setup() -> bool {
    false
}
#[inline]
fn default_tonemapper() -> Tonemapper {
    Tonemapper::HybridLogGamma
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_app_name")]
    pub app_name: String,
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
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_display_luminance")]
    pub display_luminance: u32,
    #[serde(default = "default_timing_setup")]
    pub timing_setup: bool,
    #[serde(default = "default_tonemapper")]
    pub tonemapper: Tonemapper,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            app_name: default_app_name(),
            major_version: default_major_version(),
            minor_version: default_minor_version(),
            patch_version: default_patch_version(),
            asset_path: default_asset_path(),
            vulkan_debug_output: default_vulkan_debug_output(),
            vulkan_log_level: default_vulkan_log_level(),
            vulkan_layers: default_vulkan_layers(),
            fps_cap: default_fps_cap(),
            reversed_depth_buffer: default_reversed_depth_buffer(),
            width: default_width(),
            height: default_height(),
            display_luminance: default_display_luminance(),
            timing_setup: default_timing_setup(),
            tonemapper: default_tonemapper(),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "    App name: {}", self.app_name)?;
        writeln!(
            f,
            "    App version: {}.{}.{}",
            self.major_version, self.minor_version, self.patch_version
        )?;
        writeln!(
            f,
            "    Renderer version: {}.{}.{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH")
        )?;
        writeln!(f, "    Asset path: {:?}", self.asset_path)?;
        writeln!(f, "    Vulkan debug output: {:?}", self.vulkan_debug_output)?;
        writeln!(f, "    Vulkan log level: {:?}", self.vulkan_log_level)?;
        writeln!(f, "    Vulkan log layers:")?;
        for layer in &self.vulkan_layers {
            writeln!(f, "      {}", layer)?;
        }
        writeln!(
            f,
            "    Reversed depth buffer: {:?}",
            self.reversed_depth_buffer
        )?;
        writeln!(f, "    FPS cap: {}", self.fps_cap)?;
        writeln!(f, "    width: {}", self.width)?;
        writeln!(f, "    height: {}", self.height)?;
        writeln!(
            f,
            "    display luminance (max): {} cd/m²",
            self.display_luminance
        )?;
        writeln!(f, "    Timing Setup: {}", self.timing_setup)?;
        writeln!(f, "    Tone mapper: {:?}", self.tonemapper)?;
        Ok(())
    }
}
