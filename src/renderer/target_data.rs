
use dacite::core::{Device, Extent2D, CommandBuffer, ImageLayout, AccessFlags,
                   PipelineStageFlags, ImageAspectFlags, OptionalMipLevels,
                   OptionalArrayLayers, ImageSubresourceRange};
use errors::*;
use super::image_wrap::{ImageWrap, ImageWrapType};
use super::memory::{Memory, Lifetime};
use super::commander::Commander;
use super::setup::requirements::{DEPTH_FORMAT,
                                 DIFFUSE_FORMAT,
                                 NORMALS_FORMAT,
                                 MATERIAL_FORMAT,
                                 SHADING_FORMAT,
                                 BLUR_FORMAT};

/*
Depth:			D32_SFloat
Diffuse:		A2B10G10R10_UNorm_Pack32
Normal:			A2B10G10R10_UNorm_Pack32
Material:		R8G8B8A8_UNorm
  r-channel is used for "roughness"
  g-channel is used for "metallicity"
  b-channel is used for "ambient occlusion"
  a-channel unused
Shading:                R16G16B16A16_SFloat (goes overbright)
Blur:                   R16G16B16A16_SFloat (goes overbright)
 */

pub struct TargetData {
    pub blur_image: ImageWrap,
    pub shading_image: ImageWrap,
    pub material_image: ImageWrap,
    pub normals_image: ImageWrap,
    pub diffuse_image: ImageWrap,
    pub depth_image: ImageWrap,
    pub extent: Extent2D
}

impl TargetData {
    pub fn create(device: &Device,
                  memory: &mut Memory,
                  commander: &Commander,
                  extent: Extent2D)
                  -> Result<TargetData>
    {
        let (depth_image, diffuse_image, normals_image, material_image,
             shading_image, blur_image) =
            build_images(device, memory, commander, extent)?;

        Ok(TargetData {
            blur_image: blur_image,
            shading_image: shading_image,
            material_image: material_image,
            normals_image: normals_image,
            diffuse_image: diffuse_image,
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
        let (depth_image, diffuse_image, normals_image, material_image,
             shading_image, blur_image) =
            build_images(device, memory, commander, extent)?;
        self.depth_image = depth_image;
        self.diffuse_image = diffuse_image;
        self.normals_image = normals_image;
        self.material_image = material_image;
        self.shading_image = shading_image;
        self.blur_image = blur_image;

        Ok(())
    }

    pub fn transition_for_geometry(&mut self, command_buffer: CommandBuffer)
                                   -> Result<()>
    {
        // write and read depth: depth never needs transition

        // write diffuse, normal, and material
        self.diffuse_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::Undefined, ImageLayout::ColorAttachmentOptimal,
            Default::default(), AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;
        self.normals_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::Undefined, ImageLayout::ColorAttachmentOptimal,
            Default::default(), AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE, PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;
        self.material_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::Undefined, ImageLayout::ColorAttachmentOptimal,
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

    pub fn transition_for_shading(&mut self, command_buffer: CommandBuffer)
                                  -> Result<()>
    {
        // Transition depth buffer for shader reads
        self.depth_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::DepthStencilAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            AccessFlags::SHADER_READ,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // read diffuse, normal, and material
        self.diffuse_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;
        self.normals_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;
        self.material_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // write shading
        self.shading_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::Undefined, ImageLayout::ColorAttachmentOptimal,
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

    pub fn transition_for_transparent(&mut self, command_buffer: CommandBuffer)
                                   -> Result<()>
    {
        // Reinstate the depth buffer
        self.depth_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ShaderReadOnlyOptimal,
            ImageLayout::DepthStencilAttachmentOptimal,
            AccessFlags::SHADER_READ,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            PipelineStageFlags::FRAGMENT_SHADER,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;


        // write shading: already there.

        Ok(())
    }

    pub fn transition_for_blurh(&mut self, command_buffer: CommandBuffer)
                                 -> Result<()>
    {
        // read shading:
        self.shading_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // write blur:
        self.blur_image.transition_layout(
            command_buffer,
            ImageLayout::Undefined, ImageLayout::ColorAttachmentOptimal,
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

    pub fn transition_for_blurv(&mut self, command_buffer: CommandBuffer)
                                -> Result<()>
    {
        // read blur:
        self.blur_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::COLOR_ATTACHMENT_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, PipelineStageFlags::FRAGMENT_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(1),
            })?;

        // write shading:
        self.shading_image.transition_layout(
            command_buffer,
            ImageLayout::ShaderReadOnlyOptimal, ImageLayout::ColorAttachmentOptimal,
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
        // read shading:
        self.shading_image.transition_layout(
            command_buffer.clone(),
            ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal,
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
    -> Result<(ImageWrap, ImageWrap, ImageWrap, ImageWrap, ImageWrap, ImageWrap)>
{
    use dacite::core::{ComponentMapping, ImageUsageFlags, ImageLayout, ImageTiling,
                       AccessFlags, PipelineStageFlags, ImageAspectFlags,
                       OptionalMipLevels, OptionalArrayLayers, Extent3D,
                       ImageSubresourceRange};

    let mut make = |format,iwtype,usage,name| {
        ImageWrap::new(
            device,memory,format,
            ComponentMapping::identity(),
            Extent3D { width: extent.width, height: extent.height, depth: 1 },
            iwtype,
            ImageLayout::Undefined,
            ImageTiling::Optimal,
            usage,
            Lifetime::Permanent,
            name)
    };

    let depth_image = {
        let mut depth_image_wrap = make(
            DEPTH_FORMAT, ImageWrapType::Depth,
            ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
                | ImageUsageFlags::SAMPLED,
            "Depth Buffer")?;

        depth_image_wrap.transition_layout_now(
            device,
            ImageLayout::Undefined, ImageLayout::DepthStencilAttachmentOptimal,
            Default::default(),
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
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

    let diffuse_image = make(
        DIFFUSE_FORMAT, ImageWrapType::Standard,
        ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
            | ImageUsageFlags::SAMPLED,
        "Diffuse g-buffer")?;

    let normals_image = make(
        NORMALS_FORMAT, ImageWrapType::Standard,
        ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
            | ImageUsageFlags::SAMPLED,
        "Normals g-buffer")?;

    let material_image = make(
        MATERIAL_FORMAT, ImageWrapType::Standard,
        ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
            | ImageUsageFlags::SAMPLED,
        "Materials g-buffer")?;

    let shading_image = make(
        SHADING_FORMAT, ImageWrapType::Standard,
        ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
            | ImageUsageFlags::SAMPLED,
        "Shading Target")?;

    let blur_image = make(
        BLUR_FORMAT, ImageWrapType::Standard,
        ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::INPUT_ATTACHMENT
            | ImageUsageFlags::SAMPLED,
        "Blur Target")?;

    Ok((depth_image, diffuse_image, normals_image, material_image,
        shading_image, blur_image))
}
