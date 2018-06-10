use winit::Window;

pub mod debug_report;
pub mod instance;
pub mod requirements;
pub mod surface;

#[allow(dead_code)]
enum SurfaceKind {
    Xlib,
    Xcb,
    Wayland,
    Win32,
    Android,
}

fn get_surface_kind(window: &Window) -> SurfaceKind {
    #[cfg(all(unix, not(target_os = "android")))]
    {
        use winit::os::unix::WindowExt;
        if window.get_wayland_display().is_some() {
            return SurfaceKind::Wayland;
        } else if window.get_xlib_display().is_some() {
            return SurfaceKind::Xlib;
        } /* else if window.get_xcb_connection().is_some() {
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
