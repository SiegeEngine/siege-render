use ash::extensions::Surface;
use ash::vk::types::{ColorSpaceKHR, Extent2D, Format, PhysicalDevice, PresentModeKHR,
                     SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR};
use errors::*;

pub struct SurfaceData {
    pub capabilities: SurfaceCapabilitiesKHR,
    // index into surface_formats for the format we are using
    pub surface_format_index: usize,
    pub surface_formats: Vec<SurfaceFormatKHR>,
    pub min_image_count: u32,
    pub present_mode: PresentModeKHR,
    pub needs_gamma: bool,
    // TODO: SurfaceTransformFlagsKHR
    // TODO: ImageUsageFlags
}

impl SurfaceData {
    pub fn create(
        physical_device: PhysicalDevice,
        surface_khr: SurfaceKHR,
        surface: &Surface,
    ) -> Result<SurfaceData> {
        let capabilities =
            surface.get_physical_device_surface_capabilities_khr(physical_device, surface_khr)?;
        let surface_formats: Vec<SurfaceFormatKHR> =
            surface.get_physical_device_surface_formats_khr(physical_device, surface_khr)?;
        let min_image_count = {
            use std::cmp;
            cmp::max(
                capabilities.min_image_count,
                cmp::min(3, capabilities.max_image_count),
            )
        };
        let present_mode = {
            let present_modes = surface
                .get_physical_device_surface_present_modes_khr(physical_device, surface_khr)?;
            get_present_mode(&present_modes)
        };

        // Choose the best surface format available
        let ranking = |f: Format| -> u32 {
            match f {
                Format::A2b10g10r10UnormPack32 => 3, // best due to color depth
                Format::B8g8r8a8Srgb => 2,           // good due to auto-srgb conversion
                Format::B8g8r8a8Unorm => 1,          // accepable
                _ => 0,
            }
        };
        let mut surface_format_index: Option<usize> = None;
        for i in 0..surface_formats.len() {
            //println!("Offered: {:?}", surface_formats[i].format);
            if let Some(sfi) = surface_format_index {
                if ranking(surface_formats[i].format) > ranking(surface_formats[sfi].format) {
                    surface_format_index = Some(i);
                }
            } else {
                if ranking(surface_formats[i].format) > 0 {
                    surface_format_index = Some(i);
                }
            }
        }
        let surface_format_index = match surface_format_index {
            Some(i) => i,
            None => return Err(ErrorKind::NoSuitableSurfaceFormat.into()),
        };
        info!(
            "Surface format: {:?}",
            surface_formats[surface_format_index].format
        );
        info!(
            "Surface color space: {:?}",
            surface_formats[surface_format_index].color_space
        );

        let needs_gamma = surface_formats[surface_format_index].color_space
            == ColorSpaceKHR::SrgbNonlinear
            && match surface_formats[surface_format_index].format {
                Format::A2b10g10r10UnormPack32 => true,
                Format::B8g8r8a8Srgb => false,
                Format::B8g8r8a8Unorm => true,
                _ => true,
            };

        Ok(SurfaceData {
            capabilities: capabilities,
            surface_format_index: surface_format_index,
            surface_formats: surface_formats,
            min_image_count: min_image_count,
            present_mode: present_mode,
            needs_gamma: needs_gamma,
        })
    }

    pub fn update(
        &mut self,
        physical_device: PhysicalDevice,
        surface_khr: SurfaceKHR,
        surface: &Surface,
    ) -> Result<()> {
        self.capabilities =
            surface.get_physical_device_surface_capabilities_khr(physical_device, surface_khr)?;
        Ok(())
    }

    pub fn get_surface_extent(&self, preferred_extent: Extent2D) -> Extent2D {
        self.capabilities.current_extent
    }

    #[inline]
    pub fn format(&self) -> Format {
        self.surface_formats[self.surface_format_index].format
    }

    #[inline]
    pub fn color_space(&self) -> ColorSpaceKHR {
        self.surface_formats[self.surface_format_index].color_space
    }
}

fn get_present_mode(present_modes: &Vec<PresentModeKHR>) -> PresentModeKHR {
    present_modes
        .iter()
        .map(|mode| *mode)
        .min_by_key(|mode| match *mode {
            PresentModeKHR::Mailbox => 1,
            PresentModeKHR::FifoRelaxed => 2,
            PresentModeKHR::Fifo => 3,
            _ => 99,
        })
        .unwrap() // Vulkan guarantees Fifo exists
}
