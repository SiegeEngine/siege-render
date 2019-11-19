
use dacite::core::{Device, RenderPass, Framebuffer, Extent2D, ImageView,
                   CommandBuffer};
use error::Error;
use renderer::image_wrap::ImageWrap;

pub struct BlurVPass {
    pub framebuffer: Framebuffer,
    pub shading_image_view: ImageView,
    pub blur_image_view: ImageView,
    pub extent: Extent2D,
    pub render_pass: RenderPass,
}

impl BlurVPass {
    pub fn new(
        device: &Device,
        blur_image: &ImageWrap,
        shading_image: &ImageWrap)
        -> Result<BlurVPass, Error>
    {
        let render_pass = {
            use dacite::core::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout,
                               SubpassDescription, SubpassDescriptionFlags,
                               PipelineBindPoint, SubpassIndex, SubpassDependency,
                               PipelineStageFlags, AccessFlags, DependencyFlags,
                               RenderPassCreateFlags, RenderPassCreateInfo,
                               AttachmentReference, AttachmentIndex};

            let blur_attachment_description = blur_image.get_attachment_description(
                AttachmentLoadOp::Load,
                AttachmentStoreOp::DontCare,
                ImageLayout::ShaderReadOnlyOptimal,
                ImageLayout::ShaderReadOnlyOptimal,
            );

            let blur_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(0),
                layout: ImageLayout::ShaderReadOnlyOptimal
            };

            let shading_attachment_description = shading_image.get_attachment_description(
                AttachmentLoadOp::Load,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );

            let shading_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(1),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let subpass = SubpassDescription {
                flags: SubpassDescriptionFlags::empty(),
                pipeline_bind_point: PipelineBindPoint::Graphics,
                input_attachments: vec![blur_attachment_reference],
                color_attachments: vec![shading_attachment_reference],
                resolve_attachments: vec![],
                depth_stencil_attachment: None,
                preserve_attachments: vec![],
            };

            // We must have written the shading image before this RenderPass reads it
            let blur_h_to_blur_v = SubpassDependency {
                src_subpass: SubpassIndex::External, // blur_h (prior pass)
                dst_subpass: SubpassIndex::Index(0), // us
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            // We must write the shading image before the next RenderPass reads it
            let blur_v_to_post = SubpassDependency {
                src_subpass: SubpassIndex::Index(0), // us
                dst_subpass: SubpassIndex::External, // post
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            let create_info = RenderPassCreateInfo {
                flags: RenderPassCreateFlags::empty(),
                attachments: vec![
                    blur_attachment_description,
                    shading_attachment_description,
                ],
                subpasses: vec![subpass],
                dependencies: vec![
                    blur_h_to_blur_v,
                    blur_v_to_post
                ],
                chain: None,
            };

            device.create_render_pass(&create_info, None)?
        };

        let (blur_image_view, shading_image_view, framebuffer, extent) =
            build(device, render_pass.clone(), blur_image, shading_image)?;

        Ok(BlurVPass {
            framebuffer: framebuffer,
            shading_image_view: shading_image_view,
            blur_image_view: blur_image_view,
            extent: extent,
            render_pass: render_pass,
        })
    }

    pub fn rebuild(&mut self, device: &Device,
                   blur_image: &ImageWrap,
                   shading_image: &ImageWrap)
                   -> Result<(), Error>
    {
        let (blur_image_view, shading_image_view, framebuffer, extent) =
            build(device, self.render_pass.clone(), blur_image, shading_image)?;

        self.framebuffer = framebuffer;
        self.blur_image_view = blur_image_view;
        self.shading_image_view = shading_image_view;
        self.extent = extent;

        Ok(())
    }

    pub fn record_entry(&self, command_buffer: CommandBuffer)
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
    }

    pub fn record_exit(&self, command_buffer: CommandBuffer)
    {
        command_buffer.end_render_pass();
    }
}

fn build(device: &Device, render_pass: RenderPass, blur_image: &ImageWrap,
         shading_image: &ImageWrap)
    -> Result<(ImageView, ImageView, Framebuffer, Extent2D), Error>
{
    let blur_image_view = blur_image.get_image_view(device)?;

    let shading_image_view = shading_image.get_image_view(device)?;

    let extent = Extent2D {
        width: blur_image.extent.width,
        height: blur_image.extent.height
    };

    let framebuffer = {
        use dacite::core::{FramebufferCreateInfo, FramebufferCreateFlags};

        let create_info = FramebufferCreateInfo {
            flags: FramebufferCreateFlags::empty(),
            render_pass: render_pass,
            attachments: vec![
                blur_image_view.clone(),
                shading_image_view.clone(),
            ],
            width: extent.width,
            height: extent.height,
            layers: 1,
            chain: None,
        };
        device.create_framebuffer(&create_info, None)?
    };

    Ok((blur_image_view, shading_image_view, framebuffer, extent))
}
