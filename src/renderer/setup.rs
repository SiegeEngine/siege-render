
use std::sync::Arc;

use dacite::core::{InstanceExtensions, Instance};
use dacite::ext_debug_report::{DebugReportFlagsExt, DebugReportObjectTypeExt,
                               DebugReportCallbackExt, DebugReportCallbacksExt};
use dacite::khr_surface::SurfaceKhr;
use dacite_winit::WindowExt;
use winit::Window;

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
