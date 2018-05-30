use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::types::{StructureType, SurfaceKHR};
use errors::*;
use std::ptr;
use winit::Window;

use super::SurfaceKind;

pub fn setup_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &Window,
) -> Result<SurfaceKHR> {
    match super::get_surface_kind(window) {
        SurfaceKind::Xlib => {
            #[cfg(all(unix, not(target_os = "android")))]
            {
                use ash::extensions::XlibSurface;
                use ash::vk::types::{Display, XlibSurfaceCreateInfoKHR};
                use winit::os::unix::WindowExt;
                let x11_display = window.get_xlib_display().unwrap();
                let x11_window = window.get_xlib_window().unwrap();
                let x11_create_info = XlibSurfaceCreateInfoKHR {
                    s_type: StructureType::XlibSurfaceCreateInfoKhr,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    window: x11_window as ::ash::vk::types::Window,
                    dpy: x11_display as *mut Display,
                };
                let xlib_surface_loader = XlibSurface::new(entry, instance)?;
                Ok(unsafe { xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None) }?)
            }
            #[cfg(not(all(unix, not(target_os = "android"))))]
            {
                panic!("Surface is xlib, but os does not match!");
            }
        }
        SurfaceKind::Xcb => {
            #[cfg(all(unix, not(target_os = "android")))]
            {
                use ash::extensions::XcbSurface;
                use ash::vk::types::{xcb_connection_t, XcbSurfaceCreateInfoKHR};
                use winit::os::unix::WindowExt;
                let xcb_connection = window.get_xcb_connection().unwrap() as *mut xcb_connection_t;
                let xcb_window: u32 = 0;
                let xcb_create_info = XcbSurfaceCreateInfoKHR {
                    s_type: StructureType::XcbSurfaceCreateInfoKhr,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    connection: xcb_connection,
                    window: xcb_window,
                };
                let xcb_surface_loader = XcbSurface::new(entry, instance)?;
                Ok(unsafe { xcb_surface_loader.create_xcb_surface_khr(&xcb_create_info, None) }?)
            }
            #[cfg(not(all(unix, not(target_os = "android"))))]
            {
                panic!("Surface is xcb, but os does not match!");
            }
        }
        SurfaceKind::Wayland => unimplemented!(),
        SurfaceKind::Win32 => {
            #[cfg(windows)]
            {
                use ash::extensions::Win32Surface;
                use ash::vk::types::Win32SurfaceCreateInfoKHR;
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
                let win32_surface_loader = Win32Surface::new(entry, instance)?;
                Ok(unsafe {
                    win32_surface_loader.create_win32_surface_khr(&win32_create_info, None)
                }?)
            }
            #[cfg(not(windows))]
            {
                panic!("Surface is win32, but os does not match!");
            }
        }
        SurfaceKind::Android => unimplemented!(),
    }
}
