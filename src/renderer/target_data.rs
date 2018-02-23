
use dacite::core::{Device, Extent2D, CommandBuffer, ImageLayout, AccessFlags,
                   PipelineStageFlags, ImageAspectFlags, OptionalMipLevels,
                   OptionalArrayLayers, ImageSubresourceRange};
use errors::*;
use super::image_wrap::{ImageWrap, ImageWrapType};
use super::memory::{Memory, Lifetime};
use super::commander::Commander;
use super::setup::requirements::{DEPTH_FORMAT, SHADING_IMAGE_FORMAT,
                                 BRIGHT_AND_BLUR_IMAGE_FORMAT};

pub struct TargetData {
    pub blurpong_image: ImageWrap,
    pub bright_image: ImageWrap,
    pub shading_image: ImageWrap,
    pub depth_image: ImageWrap,
    pub extent: Extent2D,
}

impl TargetData {
    pub fn create(device: &Device,
                  memory: &mut Memory,
                  commander: &Commander,
                  extent: Extent2D)
                  -> Result<TargetData>
    {
        let (depth_image, shading_image, bright_image, blurpong_image) =
            build_images(device, memory, commander, extent)?;

        Ok(TargetData {
            blurpong_image: blurpong_image,
            bright_image: bright_image,
            shading_image: shading_image,
            depth_image: depth_image,
            extent: extent
        })
    }

    pub fn rebuild(&mut self,
                   device: &Device,
                   memory: &mut Memory,
                   commander: &Commander,
                   extent: Extent2D)
                   -> Result<()>
    {
        self.extent = extent;

        // Rebuild images
        let (depth_image, shading_image, bright_image, blurpong_image) =
            build_images(device, memory, commander, extent)?;
        self.depth_image = depth_image;
        self.shading_image = shading_image;
        self.bright_image = bright_image;
        self.blurpong_image = blurpong_image;

        Ok(())
    }

    pub fn transition_for_earlyz(&mut self, _command_buffer: CommandBuffer)
                                   -> Result<()>
    {
        // write depth: depth never needs transition.

        Ok(())
    }

    pub fn transition_for_opaque(&mut self, command_buffer: CommandBuffer)
                                 -> Result<()>
    {
        // read depth: depth never needs transition.

        // write shading:
        self.shading_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal,
            Default::default(), AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // Nothing to transition, already there.
        Ok(())
    }

    pub fn transition_for_transparent(&mut self, _command_buffer: CommandBuffer)
                                   -> Result<()>
    {
        // read depth: depth never needs transition.

        // write shading: already there.

        Ok(())
    }

    pub fn transition_for_bloom_filter(&mut self, command_buffer: CommandBuffer)
                                        -> Result<()>
    {
        // read shading:
        self.shading_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // write bright:
        self.bright_image.transition_layout(
            command_buffer,
            ImageLayout::ColorAttachmentOptimal,
            Default::default(), AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        Ok(())
    }

    pub fn transition_for_bloom_h(&mut self, command_buffer: CommandBuffer)
                                  -> Result<()>
    {
        // read bright:
        self.bright_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // write blurpong
        self.blurpong_image.transition_layout(
            command_buffer,
            ImageLayout::ColorAttachmentOptimal,
            Default::default(), AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        Ok(())
    }

    pub fn transition_for_bloom_v(&mut self, command_buffer: CommandBuffer)
                                    -> Result<()>
    {
        // read blurpong:
        self.blurpong_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // write bright:
        self.bright_image.transition_layout(
            command_buffer,
            ImageLayout::ColorAttachmentOptimal,
            AccessFlags::SHADER_READ, AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::FRAGMENT_SHADER, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        Ok(())
    }

    pub fn transition_for_post(&mut self, command_buffer: CommandBuffer)
                               -> Result<()>
    {
        // read shading: already there

        // read bright:
        self.bright_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // (write swapchain): not handled here

        Ok(())
    }

    pub fn transition_for_ui(&mut self, _command_buffer: CommandBuffer)
                             -> Result<()>
    {
        // (write swapchain): not handled here

        Ok(())
    }
}

fn build_images(
    device: &Device,
    memory: &mut Memory,
    commander: &Commander,
    extent: Extent2D)
    -> Result<(ImageWrap, ImageWrap, ImageWrap, ImageWrap)>
{
    use dacite::core::{ComponentMapping, ImageUsageFlags, ImageLayout, ImageTiling,
                       AccessFlags, PipelineStageFlags, ImageAspectFlags,
                       OptionalMipLevels, OptionalArrayLayers, Extent3D,
                       ImageSubresourceRange};

    let depth_image = {
        let mut depth_image_wrap = ImageWrap::new(
            device,
            memory,
            DEPTH_FORMAT,
            ComponentMapping::identity(),
            Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            ImageWrapType::Depth,
            ImageLayout::Undefined,
            ImageTiling::Optimal,
            ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            Lifetime::Permanent,
            "Depth Buffer")?;

        depth_image_wrap.transition_layout_now(
            device,
            ImageLayout::DepthStencilAttachmentOptimal,
            Default::default(),
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            },
            commander
        )?;

        /* FIXME:
        Consider VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR,
        which allows use as a depth-stencil attachment where depth is read-only
        which is useful for shading phase - allows ImageUsage::Sampled along
        with ImageUsage::DepthStencilAttachment
         */

        depth_image_wrap
    };

    let shading_image = {
        let mut shading_image_wrap = ImageWrap::new(
            device,
            memory,
            SHADING_IMAGE_FORMAT,
            ComponentMapping::identity(),
            Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            ImageWrapType::Standard,
            ImageLayout::Undefined,
            ImageTiling::Optimal,
            ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
                | ImageUsageFlags::SAMPLED,
            Lifetime::Permanent,
            "Shading Image")?;

        shading_image_wrap.transition_layout_now(
            device,
            ImageLayout::ColorAttachmentOptimal,
            Default::default(),
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            },
            commander
        )?;

        shading_image_wrap
    };

    let bright_image = {
        let mut bright_image_wrap = ImageWrap::new(
            device,
            memory,
            BRIGHT_AND_BLUR_IMAGE_FORMAT,
            ComponentMapping::identity(),
            Extent3D {
                // FIXME: can we have half dimensions?
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            ImageWrapType::Standard,
            ImageLayout::Undefined,
            ImageTiling::Optimal,
            ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
                | ImageUsageFlags::SAMPLED,
            Lifetime::Permanent,
            "bright Image")?;

        bright_image_wrap.transition_layout_now(
            device,
            ImageLayout::ColorAttachmentOptimal,
            Default::default(),
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            },
            commander
        )?;

        bright_image_wrap
    };

    let blurpong_image = {
        let mut blurpong_image_wrap = ImageWrap::new(
            device,
            memory,
            BRIGHT_AND_BLUR_IMAGE_FORMAT,
            ComponentMapping::identity(),
            Extent3D {
                // FIXME can we have half dimensions?
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            ImageWrapType::Standard,
            ImageLayout::Undefined,
            ImageTiling::Optimal,
            ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
                | ImageUsageFlags::SAMPLED,
            Lifetime::Permanent,
            "Blur-pong Image")?;

        blurpong_image_wrap.transition_layout_now(
            device,
            ImageLayout::ColorAttachmentOptimal,
            Default::default(),
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            },
            commander
        )?;

        blurpong_image_wrap
    };

    Ok((depth_image, shading_image, bright_image, blurpong_image))
}
