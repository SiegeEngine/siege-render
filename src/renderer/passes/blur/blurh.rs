
use dacite::core::{Device, RenderPass, Framebuffer, Extent2D, ImageView,
                   CommandBuffer};
use errors::*;
use renderer::image_wrap::ImageWrap;

pub struct BlurHPass {
    pub framebuffer: Framebuffer,
    pub blur_image_view: ImageView,
    pub shading_image_view: ImageView,
    pub extent: Extent2D,
    pub render_pass: RenderPass,
}

impl BlurHPass {
    pub fn new(
        device: &Device,
        shading_image: &ImageWrap,
        blur_image: &ImageWrap)
        -> Result<BlurHPass>
    {
        let render_pass = {
            use dacite::core::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout,
                               SubpassDescription, SubpassDescriptionFlags,
                               PipelineBindPoint, SubpassIndex, SubpassDependency,
                               PipelineStageFlags, AccessFlags, DependencyFlags,
                               RenderPassCreateFlags, RenderPassCreateInfo,
                               AttachmentReference, AttachmentIndex};

            let shading_attachment_description = shading_image.get_attachment_description(
                AttachmentLoadOp::Load,
                AttachmentStoreOp::Store,
                ImageLayout::ShaderReadOnlyOptimal,
                ImageLayout::ShaderReadOnlyOptimal,
            );

            let shading_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(0),
                layout: ImageLayout::ShaderReadOnlyOptimal
            };

            let blur_attachment_description = blur_image.get_attachment_description(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );

            let blur_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(1),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let subpass = SubpassDescription {
                flags: SubpassDescriptionFlags::empty(),
                pipeline_bind_point: PipelineBindPoint::Graphics,
                input_attachments: vec![shading_attachment_reference],
                color_attachments: vec![blur_attachment_reference],
                resolve_attachments: vec![],
                depth_stencil_attachment: None,
                preserve_attachments: vec![],
            };

            // We must have written the shading image before this RenderPass reads it
            let transparent_to_blurh = SubpassDependency {
                src_subpass: SubpassIndex::External, // blur_filter (prior pass)
                dst_subpass: SubpassIndex::Index(0), // us
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            // We must write the blur image before the next RenderPass reads it
            let blurh_to_blurv = SubpassDependency {
                src_subpass: SubpassIndex::Index(0), // us
                dst_subpass: SubpassIndex::External, // blur_h
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            let create_info = RenderPassCreateInfo {
                flags: RenderPassCreateFlags::empty(),
                attachments: vec![
                    shading_attachment_description,
                    blur_attachment_description,
                ],
                subpasses: vec![subpass],
                dependencies: vec![
                    transparent_to_blurh,
                    blurh_to_blurv,
                ],
                chain: None,
            };

            device.create_render_pass(&create_info, None)?
        };

        let (shading_image_view, blur_image_view, framebuffer, extent) =
            build(device, render_pass.clone(), shading_image, blur_image)?;

        Ok(BlurHPass {
            framebuffer: framebuffer,
            blur_image_view: blur_image_view,
            shading_image_view: shading_image_view,
            extent: extent,
            render_pass: render_pass,
        })
    }

    pub fn rebuild(&mut self, device: &Device,
                   shading_image: &ImageWrap,
                   blur_image: &ImageWrap)
                   -> Result<()>
    {
        let (shading_image_view, blur_image_view, framebuffer, extent) =
            build(device, self.render_pass.clone(), shading_image, blur_image)?;

        self.framebuffer = framebuffer;
        self.shading_image_view = shading_image_view;
        self.blur_image_view = blur_image_view;
        self.extent = extent;

        Ok(())
    }

    pub fn record_entry(&self,
                        command_buffer: CommandBuffer)
                        -> Result<()>
    {
        use dacite::core::{Rect2D, Offset2D,
                           SubpassContents, RenderPassBeginInfo,
                           ClearValue, ClearColorValue};

        let begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            framebuffer: self.framebuffer.clone(),
            render_area: Rect2D::new(Offset2D::zero(), self.extent),
            clear_values:  vec![
                ClearValue::Color( // unused
                    ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])),
                ClearValue::Color(
                    ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])),
            ],
            chain: None,
        };

        command_buffer.begin_render_pass(
            &begin_info, SubpassContents::Inline);

        Ok(())
    }

    pub fn record_exit(
        &self,
        command_buffer: CommandBuffer) -> Result<()>
    {
        command_buffer.end_render_pass();

        Ok(())
    }
}

fn build(device: &Device, render_pass: RenderPass, shading_image: &ImageWrap,
         blur_image: &ImageWrap)
    -> Result<(ImageView, ImageView, Framebuffer, Extent2D)>
{
    let shading_image_view = shading_image.get_image_view(device)?;

    let blur_image_view = blur_image.get_image_view(device)?;

    let extent = Extent2D {
        width: shading_image.extent.width,
        height: shading_image.extent.height
    };

    let framebuffer = {
        use dacite::core::{FramebufferCreateInfo, FramebufferCreateFlags};

        let create_info = FramebufferCreateInfo {
            flags: FramebufferCreateFlags::empty(),
            render_pass: render_pass,
            attachments: vec![
                shading_image_view.clone(),
                blur_image_view.clone(),
            ],
            width: extent.width,
            height: extent.height,
            layers: 1,
            chain: None,
        };
        device.create_framebuffer(&create_info, None)?
    };

    Ok((shading_image_view, blur_image_view, framebuffer, extent))
}
