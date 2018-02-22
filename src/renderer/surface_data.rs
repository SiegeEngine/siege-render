
use errors::*;
use dacite::core::{PhysicalDevice, Extent2D, Format};
use dacite::khr_surface::{SurfaceKhr, SurfaceCapabilitiesKhr,
                          SurfaceFormatKhr, PresentModeKhr,
                          ColorSpaceKhr};

pub struct SurfaceData {
    pub capabilities: SurfaceCapabilitiesKhr,
    pub surface_formats: Vec<SurfaceFormatKhr>,
    pub min_image_count: u32,
    pub present_mode: PresentModeKhr,

    // TODO: SurfaceTransformFlagsKhr
    // TODO: ImageUsageFlags
}

impl SurfaceData {

    pub fn create(physical_device: &PhysicalDevice,
                  surface: &SurfaceKhr,
                  vsync: bool)
                  -> Result<SurfaceData>
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
            get_present_mode(vsync, &present_modes)
        };

        Ok(SurfaceData {
            capabilities: capabilities,
            surface_formats: surface_formats,
            min_image_count: min_image_count,
            present_mode: present_mode,
        })
    }

    pub fn update(&mut self,
                  physical_device: &PhysicalDevice,
                  surface: &SurfaceKhr) -> Result<()>
    {
        self.capabilities = physical_device.get_surface_capabilities_khr(surface)?;
        Ok(())
    }

    pub fn require_surface_format(&self, format: Format, color_space: ColorSpaceKhr)
                                  -> Result<()>
    {
        for surface_format in &self.surface_formats {
            if surface_format.format == format &&
                surface_format.color_space == color_space
            {
                return Ok(())
            }
        }
        Err(ErrorKind::NoSuitableSurfaceFormat.into())
    }

    pub fn get_surface_extent(&self, preferred_extent: Extent2D) -> Extent2D
    {
        match self.capabilities.current_extent {
            Some(extent) => extent,
            None => preferred_extent
        }
    }
}

fn get_present_mode(vsync: bool, present_modes: &Vec<PresentModeKhr>) -> PresentModeKhr
{
    if vsync {
        present_modes.iter().map(|mode| *mode).min_by_key(|mode| {
            match *mode {
                PresentModeKhr::Mailbox => 1,
                PresentModeKhr::Fifo => 2,
                _ => 99,
            }
        }).unwrap() // Vulkan guarantees Fifo exists
    } else {
        present_modes.iter().map(|mode| *mode).min_by_key(|mode| {
            match *mode {
                PresentModeKhr::Immediate => 1,
                PresentModeKhr::Mailbox => 2,
                PresentModeKhr::FifoRelaxed => 3,
                PresentModeKhr::Fifo => 4,
                _ => 99,
            }
        }).unwrap() // Vulkan guarantees Fifo exists
    }
}
