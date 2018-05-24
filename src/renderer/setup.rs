use ash::version::{EntryV1_0, V1_0};
use ash::vk::types::StructureType;
use ash::{Entry, Instance};
use config::Config;
use errors::*;
use siege_vulkan::CStringSet;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use winit::Window;

pub fn setup_instance(
    entry: &Entry<V1_0>,
    config: &Config,
    window: &Window,
) -> Result<Instance<V1_0>> {
    use ash::vk::types::{ApplicationInfo, InstanceCreateInfo};

    let app_name = CString::new(&*config.app_name)?;
    let engine_name = CString::new("Siege Engine")?;

    let app_info = ApplicationInfo {
        s_type: StructureType::ApplicationInfo,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: vk_make_version!(
            config.major_version,
            config.minor_version,
            config.patch_version
        ),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk_make_version!(
            env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap(),
            env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap(),
            env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap()
        ),
        api_version: vk_make_version!(1, 0, 3),
    };

    let layers = CStringSet::new(config.vulkan_layers.iter().map(|s| &**s).collect());
    for layer in layers.iter() {
        info!("LAYER: {}", layer.to_str().unwrap());
    }

    let extensions: CStringSet = get_extensions(entry, config, window)?;
    for extension in extensions.iter() {
        info!("EXTENSION: {}", extension.to_str().unwrap());
    }

    let create_info = InstanceCreateInfo {
        s_type: StructureType::InstanceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        p_application_info: &app_info,
        pp_enabled_layer_names: layers.pp,
        enabled_layer_count: layers.len() as u32,
        pp_enabled_extension_names: extensions.pp,
        enabled_extension_count: extensions.len() as u32,
    };

    Ok(unsafe { entry.create_instance(&create_info, None)? })
}

fn get_extensions(entry: &Entry<V1_0>, config: &Config, window: &Window) -> Result<CStringSet> {
    let mut required_extensions = get_required_surface_extensions(window);

    if config.vulkan_debug_output {
        required_extensions.push("VK_EXT_debug_report");
    }

    // Ensure all extensions we need are available
    let available_extensions_raw = entry.enumerate_instance_extension_properties()?;
    let available_extensions: Vec<&[u8]> = {
        let mut ae: Vec<&[u8]> = Vec::new();
        for ava in &available_extensions_raw {
            let a: &[u8] = unsafe {
                ::std::slice::from_raw_parts(
                    ava.extension_name.as_ptr() as *const u8,
                    ava.extension_name.iter().position(|c| *c == 0).unwrap(),
                )
            };
            debug!("Available Extension: {:?}", unsafe {
                CStr::from_ptr(a.as_ptr() as *const c_char)
            });
            ae.push(a);
        }
        ae
    };

    'outer: for req in &required_extensions {
        for ava in &available_extensions {
            if req.as_bytes() == *ava {
                continue 'outer;
            }
        }
        return Err(ErrorKind::MissingExtension(req.to_string()).into());
    }

    Ok(CStringSet::new(required_extensions))
}

fn get_required_surface_extensions(window: &Window) -> Vec<&'static str> {
    let mut required: Vec<&'static str> = Vec::new();

    required.push("VK_KHR_surface");

    #[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd",
              target_os = "openbsd"))]
    {
        use winit::os::unix::WindowExt;
        if window.get_wayland_display().is_some() {
            required.push("VK_KHR_wayland_surface");
        } else if window.get_xcb_connection().is_some() {
            required.push("VK_KHR_xcb_surface");
        } else if window.get_xlib_display().is_some() {
            required.push("VK_KHR_xlib_surface");
        } else {
            panic!("Vulkan does not have a KHR surface extension for the window provided.");
        }
        // There is also a vulkan VK_KHR_mir_surface, but winit doesn't support mir.
    }

    #[cfg(target_os = "windows")]
    {
        required.push("VK_KHR_win32_surface");
    }

    #[cfg(target_os = "android")]
    {
        required.push("VK_KHR_android_surface");
    }

    #[cfg(target_os = "macos")]
    {
        // There is no surface for mac
        panic!("Vulkan does not have a KHR surface extension for MacOS windows.")
    }

    required
}
