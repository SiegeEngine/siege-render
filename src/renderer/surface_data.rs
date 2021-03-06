
use crate::error::Error;
use dacite::core::{PhysicalDevice, Extent2D, Format};
use dacite::khr_surface::{SurfaceKhr, SurfaceCapabilitiesKhr,
                          SurfaceFormatKhr, PresentModeKhr,
                          ColorSpaceKhr};

pub struct SurfaceData {
    pub capabilities: SurfaceCapabilitiesKhr,
    // index into surface_formats for the format we are using
    pub surface_format_index: usize,
    pub surface_formats: Vec<SurfaceFormatKhr>,
    pub min_image_count: u32,
    pub present_mode: PresentModeKhr,
    pub needs_gamma: bool,
    // TODO: SurfaceTransformFlagsKhr
    // TODO: ImageUsageFlags
}

impl SurfaceData {

    pub fn create(physical_device: &PhysicalDevice,
                  surface: &SurfaceKhr)
                  -> Result<SurfaceData, Error>
    {
        let capabilities = physical_device.get_surface_capabilities_khr(surface)?;
        let surface_formats: Vec<SurfaceFormatKhr> =
            physical_device.get_surface_formats_khr(surface)?;
        let min_image_count = {
            use std::cmp;

            match capabilities.max_image_count {
                Some(max_image_count) => cmp::max(
                    capabilities.min_image_count,
                    cmp::min(3, max_image_count)
                ),
                None => cmp::max(capabilities.min_image_count, 3),
            }
        };
        let present_mode = {
            let present_modes = physical_device.get_surface_present_modes_khr(surface)?;
            get_present_mode(&present_modes)
        };

        // Choose the best surface format available
        let ranking = |f: Format| -> u32 {
            match f {
                Format::A2B10G10R10_UNorm_Pack32 => 3, // best due to color depth
                Format::B8G8R8A8_sRGB => 2, // good due to auto-srgb conversion
                Format::B8G8R8A8_UNorm => 1, // accepable
                _ => 0,
            }
        };
        let mut surface_format_index: Option<usize> = None;
        for i in 0..surface_formats.len() {
            //println!("Offered: {:?}", surface_formats[i].format);
            if let Some(sfi) = surface_format_index {
                if ranking(surface_formats[i].format) >
                    ranking(surface_formats[sfi].format)
                {
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
            None => return Err(Error::NoSuitableSurfaceFormat),
        };
        info!("Surface format: {:?}", surface_formats[surface_format_index].format);
        info!("Surface color space: {:?}", surface_formats[surface_format_index].color_space);

        let needs_gamma =
            surface_formats[surface_format_index].color_space == ColorSpaceKhr::SRGBNonLinear
            &&
            match surface_formats[surface_format_index].format {
                Format::A2B10G10R10_UNorm_Pack32 => true,
                Format::B8G8R8A8_sRGB => false,
                Format::B8G8R8A8_UNorm => true,
                _ => true
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

    pub fn update(&mut self,
                  physical_device: &PhysicalDevice,
                  surface: &SurfaceKhr) -> Result<(), Error>
    {
        self.capabilities = physical_device.get_surface_capabilities_khr(surface)?;
        Ok(())
    }

    pub fn get_surface_extent(&self, preferred_extent: Extent2D) -> Extent2D
    {
        match self.capabilities.current_extent {
            Some(extent) => extent,
            None => preferred_extent
        }
    }

    #[inline]
    pub fn format(&self) -> Format
    {
        self.surface_formats[self.surface_format_index].format
    }

    #[inline]
    pub fn color_space(&self) -> ColorSpaceKhr
    {
        self.surface_formats[self.surface_format_index].color_space
    }
}

fn get_present_mode(present_modes: &Vec<PresentModeKhr>) -> PresentModeKhr
{
    present_modes.iter().map(|mode| *mode).min_by_key(|mode| {
        match *mode {
            PresentModeKhr::Mailbox => 1,
            PresentModeKhr::FifoRelaxed => 2,
            PresentModeKhr::Fifo => 3,
            _ => 99,
        }
    }).unwrap() // Vulkan guarantees Fifo exists
}
