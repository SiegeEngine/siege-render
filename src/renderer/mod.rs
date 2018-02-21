
use errors::*;
use std::sync::Arc;
use config::Config;
use winit::Window;
use dacite::core::Instance;

mod requirements;
use self::requirements::*;

mod setup;

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

        Ok(Renderer {
            instance: instance,
            state: state,
            window: window,
            config: config
        })
    }
}
