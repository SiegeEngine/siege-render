
use dacite::core::{PhysicalDevice, Device, Extent2D, Format, SharingMode};
use dacite::khr_surface::{SurfaceKhr, ColorSpaceKhr};
use dacite::khr_swapchain::SwapchainKhr;
use error::Error;
use super::setup::QueueIndices;
use super::surface_data::SurfaceData;
use super::image_wrap::{ImageWrap, ImageWrapType};

pub struct SwapchainData {
    pub images: Vec<ImageWrap>,
    pub swapchain: SwapchainKhr,
    pub swapchain_queue_family_indices: Vec<u32>,
    pub image_sharing_mode: SharingMode,
    pub extent: Extent2D,
    pub surface_data: SurfaceData
}

impl SwapchainData {
    pub fn create(physical_device: &PhysicalDevice,
                  device: &Device,
                  surface: &SurfaceKhr,
                  preferred_extent: Extent2D,
                  queue_indices: &QueueIndices)
                  -> Result<SwapchainData, Error>
    {
        let surface_data = SurfaceData::create(physical_device, surface)?;

        let extent = surface_data.get_surface_extent(preferred_extent);

        let (image_sharing_mode, swapchain_queue_family_indices) =
            if queue_indices.graphics_family == queue_indices.present_family
        {
            (SharingMode::Exclusive, vec![])
        } else {
            (SharingMode::Concurrent,
             vec![queue_indices.graphics_family, queue_indices.present_family])
        };

        let swapchain = {
            use dacite::khr_swapchain::{SwapchainCreateInfoKhr, SwapchainCreateFlagsKhr};
            use dacite::khr_surface::CompositeAlphaFlagBitsKhr;
            use dacite::core::{ImageUsageFlags};

            let create_info = SwapchainCreateInfoKhr {
                flags: SwapchainCreateFlagsKhr::empty(),
                surface: surface.clone(),
                min_image_count: surface_data.min_image_count,
                image_format: surface_data.format(),
                image_color_space: surface_data.color_space(),
                image_extent: extent,
                image_array_layers: 1,
                image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
                image_sharing_mode: image_sharing_mode,
                queue_family_indices: swapchain_queue_family_indices.clone(),
                pre_transform: surface_data.capabilities.current_transform,
                composite_alpha: CompositeAlphaFlagBitsKhr::Opaque,
                present_mode: surface_data.present_mode,
                clipped: true,
                old_swapchain: None,
                chain: None,
            };
            device.create_swapchain_khr(&create_info, None)?
        };

        let images = build_images(&swapchain, extent, surface_data.format())?;

        Ok(SwapchainData {
            images: images,
            swapchain: swapchain,
            swapchain_queue_family_indices: swapchain_queue_family_indices,
            image_sharing_mode: image_sharing_mode,
            extent: extent,
            surface_data: surface_data,
        })
    }

    pub fn rebuild(&mut self,
                   physical_device: &PhysicalDevice,
                   device: &Device,
                   surface: &SurfaceKhr)
                   -> Result<(), Error>
    {
        // Update surface data
        self.surface_data.update(physical_device, surface)?;

        self.extent = self.surface_data.capabilities.current_extent
            .unwrap_or(Extent2D { // the 'or' shouldn't ever actually happen
                width: 1280,
                height: 1024,
            });

        // Rebuild swapchain
        self.swapchain = {
            use dacite::khr_swapchain::{SwapchainCreateInfoKhr, SwapchainCreateFlagsKhr};
            use dacite::khr_surface::CompositeAlphaFlagBitsKhr;
            use dacite::core::{ImageUsageFlags};

            let create_info = SwapchainCreateInfoKhr {
                flags: SwapchainCreateFlagsKhr::empty(),
                surface: surface.clone(),
                min_image_count: self.surface_data.min_image_count,
                image_format: self.format(),
                image_color_space: self.color_space(),
                image_extent: self.extent,
                image_array_layers: 1,
                image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
                image_sharing_mode: self.image_sharing_mode,
                queue_family_indices: self.swapchain_queue_family_indices.clone(),
                pre_transform: self.surface_data.capabilities.current_transform,
                composite_alpha: CompositeAlphaFlagBitsKhr::Opaque,
                present_mode: self.surface_data.present_mode,
                clipped: true,
                old_swapchain: Some(self.swapchain.clone()),
                chain: None,
            };
            device.create_swapchain_khr(&create_info, None)?
        };

        // Rebuild images
        self.images = build_images(
            &self.swapchain, self.extent, self.format())?;

        Ok(())
    }

    #[inline]
    pub fn format(&self) -> Format
    {
        self.surface_data.format()
    }

    #[inline]
    pub fn color_space(&self) -> ColorSpaceKhr
    {
        self.surface_data.color_space()
    }
}

fn build_images(
    swapchain: &SwapchainKhr,
    extent: Extent2D,
    format: Format)
    -> Result<Vec<ImageWrap>, Error>
{
    use dacite::core::{ComponentMapping, ImageUsageFlags, ImageTiling,
                       Extent3D};

    let images = {
        let mut images = swapchain.get_images_khr()?;

        let wraps = images.drain(..).map(|i| {
            ImageWrap {
                image: i.clone(),
                format: format,
                extent: Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1
                },
                mip_levels: 1,
                image_wrap_type: ImageWrapType::Swapchain,
                tiling: ImageTiling::Optimal,
                usage: ImageUsageFlags::COLOR_ATTACHMENT,
                size: 0,
                block: None,
                solo: None,
                swizzle: ComponentMapping::identity(),
            }
        }).collect();

        wraps
    };

    Ok(images)
}
