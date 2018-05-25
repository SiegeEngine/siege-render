use ash::extensions::DebugReport;
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};
use ash::vk::types::{DebugReportCallbackEXT, StructureType, SurfaceKHR};
use ash::{Entry, Instance};
use config::Config;
use errors::*;
use libc::c_void;
use renderer::VulkanLogLevel;
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

fn get_extensions<E: EntryV1_0>(entry: &E, config: &Config, window: &Window) -> Result<CStringSet> {
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

    match get_surface_kind(window) {
        SurfaceKind::Xlib => required.push("VK_KHR_xlib_surface"),
        SurfaceKind::Xcb => required.push("VK_KHR_xcb_surface"),
        SurfaceKind::Wayland => required.push("VK_KHR_wayland_surface"),
        SurfaceKind::Win32 => required.push("VK_KHR_win32_surface"),
        SurfaceKind::Android =>  required.push("VK_KHR_android_surface"),
    }

    required
}

pub fn setup_debug_report<E: EntryV1_0, I: InstanceV1_0>(entry: &E, config: &Config, instance: &I)
                                                         -> Result<DebugReportCallbackEXT>
{
    use ash::vk::types::{DebugReportCallbackCreateInfoEXT, DEBUG_REPORT_DEBUG_BIT_EXT,
                         DEBUG_REPORT_ERROR_BIT_EXT, DEBUG_REPORT_INFORMATION_BIT_EXT,
                         DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT, DEBUG_REPORT_WARNING_BIT_EXT};

    let debug_report = DebugReport::new(entry, instance)?;

    let flags = {
        let mut flags = DEBUG_REPORT_ERROR_BIT_EXT;
        if config.vulkan_log_level >= VulkanLogLevel::Warning {
            flags |= DEBUG_REPORT_WARNING_BIT_EXT;
        }
        if config.vulkan_log_level >= VulkanLogLevel::PerformanceWarning {
            flags |= DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT;
        }
        if config.vulkan_log_level >= VulkanLogLevel::Information {
            flags |= DEBUG_REPORT_INFORMATION_BIT_EXT;
        }
        if config.vulkan_log_level >= VulkanLogLevel::Debug {
            flags |= DEBUG_REPORT_DEBUG_BIT_EXT;
        }
        flags
    };

    let create_info = DebugReportCallbackCreateInfoEXT {
        s_type: StructureType::DebugReportCallbackCreateInfoExt,
        p_next: ptr::null(),
        flags: flags,
        pfn_callback: callback,
        p_user_data: ptr::null_mut(),
    };

    Ok(unsafe {
        debug_report.create_debug_report_callback_ext(
            &create_info,
            None, // allocation callbacks
        )
    }?)
}

use ash::vk::types::{DebugReportFlagsEXT, DebugReportObjectTypeEXT};

unsafe extern "system" fn callback(
    flags: DebugReportFlagsEXT,
    _object_type: DebugReportObjectTypeEXT,
    _object: u64,
    _location: usize,
    message_code: i32,
    _layer_prefix: *const c_char,
    message: *const c_char,
    _user_data: *mut c_void,
) -> u32 {
    use ash::vk::types::{DEBUG_REPORT_DEBUG_BIT_EXT, DEBUG_REPORT_ERROR_BIT_EXT,
                         DEBUG_REPORT_INFORMATION_BIT_EXT,
                         DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT, DEBUG_REPORT_WARNING_BIT_EXT};

    if *message != 0 {
        let cstr = CStr::from_ptr(message);
        let s = cstr.to_string_lossy();

        if flags.intersects(DEBUG_REPORT_ERROR_BIT_EXT) {
            error!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_WARNING_BIT_EXT) {
            warn!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT) {
            warn!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_INFORMATION_BIT_EXT) {
            info!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_DEBUG_BIT_EXT) {
            debug!("\r\n  vk[{}]: {}", message_code, s);
        }
    }

    0
}

#[allow(dead_code)]
enum SurfaceKind {
    Xlib,
    Xcb,
    Wayland,
    Win32,
    Android
}

fn get_surface_kind(window: &Window) -> SurfaceKind {
    #[cfg(all(unix, not(target_os = "android")))]
    {
        use winit::os::unix::WindowExt;
        if window.get_wayland_display().is_some() {
            return SurfaceKind::Wayland;
        } else if window.get_xlib_display().is_some() {
            return SurfaceKind::Xlib;
        }/* else if window.get_xcb_connection().is_some() {
            return SurfaceKind::Xcb;
        // FIXME: winit does not quite support xcb
        // https://github.com/tomaka/winit/issues/5
        // once it does, prefer xcb to xlib.
        }*/
    }

    #[cfg(target_os = "windows")]
    {
        return SurfaceKind::Win32;
    }

    #[cfg(target_os = "android")]
    {
        return SurfaceKind::Android;
    }

    panic!("Vulkan does not have a KHR surface extension for the window provided.");
}

pub fn setup_surface<E: EntryV1_0, I: InstanceV1_0>(entry: &E, instance: &I, window: &Window)
                                                    -> Result<SurfaceKHR>
{
    match get_surface_kind(window) {
        SurfaceKind::Xlib => {
            #[cfg(all(unix, not(target_os = "android")))]
            {
                use winit::os::unix::WindowExt;
                use ash::extensions::XlibSurface;
                use ash::vk::types::{Display, XlibSurfaceCreateInfoKHR};
                let x11_display = window.get_xlib_display().unwrap();
                let x11_window = window.get_xlib_window().unwrap();
                let x11_create_info = XlibSurfaceCreateInfoKHR {
                    s_type: StructureType::XlibSurfaceCreateInfoKhr,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    window: x11_window as ::ash::vk::types::Window,
                    dpy: x11_display as *mut Display,
                };
                let xlib_surface_loader =
                    XlibSurface::new(entry, instance)?;
                Ok(unsafe { xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None) }?)
            }
            #[cfg(not(all(unix, not(target_os = "android"))))]
            {
                panic!("Surface is xlib, but os does not match!");
            }
        },
        SurfaceKind::Xcb => {
            #[cfg(all(unix, not(target_os = "android")))]
            {
                use winit::os::unix::WindowExt;
                use ash::extensions::XcbSurface;
                use ash::vk::types::{XcbSurfaceCreateInfoKHR, xcb_connection_t};
                let xcb_connection = window.get_xcb_connection().unwrap() as *mut xcb_connection_t;
                let xcb_window: u32 = 0;
                let xcb_create_info = XcbSurfaceCreateInfoKHR {
                    s_type: StructureType::XcbSurfaceCreateInfoKhr,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    connection: xcb_connection,
                    window: xcb_window,
                };
                let xcb_surface_loader =
                    XcbSurface::new(entry, instance)?;
                Ok(unsafe { xcb_surface_loader.create_xcb_surface_khr(&xcb_create_info, None) }?)
            }
            #[cfg(not(all(unix, not(target_os = "android"))))]
            {
                panic!("Surface is xcb, but os does not match!");
            }
        },
        SurfaceKind::Wayland => {
            unimplemented!()
        },
        SurfaceKind::Win32 => {
            #[cfg(windows)]
            {
                use ash::vk::types::Win32SurfaceCreateInfoKHR;
                use ash::extensions::Win32Surface;
                use winapi::shared::windef::HWND;
                use winapi::um::winuser::GetWindow;
                use winit::os::windows::WindowExt;

                let hwnd = window.get_hwnd() as HWND;
                let hinstance = GetWindow(hwnd, 0) as *const vk::c_void;
                let win32_create_info = Win32SurfaceCreateInfoKHR {
                    s_type: StructureType::Win32SurfaceCreateInfoKhr,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    hinstance: hinstance,
                    hwnd: hwnd as *const vk::c_void,
                };
                let win32_surface_loader =
                    Win32Surface::new(entry, instance)?;
                Ok(unsafe { win32_surface_loader.create_win32_surface_khr(&win32_create_info, None) }?)
            }
            #[cfg(not(windows))]
            {
                panic!("Surface is win32, but os does not match!");
            }
        },
        SurfaceKind::Android => {
            unimplemented!()
        },
    }
}
