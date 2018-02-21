
mod requirements;
//use self::requirements::*;

mod setup;


use errors::*;
use std::sync::Arc;
use config::Config;
use winit::Window;
use dacite::core::Instance;
use dacite::ext_debug_report::{DebugReportFlagsExt, DebugReportObjectTypeExt,
                               DebugReportCallbackExt, DebugReportCallbacksExt};


#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum VulkanLogLevel {
    Error,
    Warning,
    PerformanceWarning,
    Information,
    Debug
}

pub struct Renderer<S> {
    #[allow(dead_code)] // We don't use this directly, FFI uses it
    debug_callback: Option<DebugReportCallbackExt>,
    #[allow(dead_code)] // This must stay alive until we shut down
    instance: Instance,
    state: Arc<S>,
    window: Arc<Window>,
    config: Arc<Config>,
}

impl<S> Renderer<S> {
    pub fn new(config: Arc<Config>, window: Arc<Window>, state: Arc<S>)
               -> Result<Renderer<S>>
    {
        let instance = {
            let create_info = {
                use dacite::core::{InstanceCreateFlags, InstanceCreateInfo,
                                   ApplicationInfo, Version};

                let mut extensions = setup::compute_instance_extensions(&window)?;

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
            Instance::create(&create_info, None)?
        };

        let debug_callback = if config.vulkan_debug_output {
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
            Some(debug_callback)
        } else {
            None
        };

        Ok(Renderer {
            debug_callback: debug_callback,
            instance: instance,
            state: state,
            window: window,
            config: config
        })
    }
}

#[derive(Debug)]
pub struct DebugCallback;

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
