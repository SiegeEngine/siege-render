
use dacite::core::{Device, RenderPass, Framebuffer, Extent2D, ImageView,
                   CommandBuffer, ClearValue};
use errors::*;
use renderer::image_wrap::ImageWrap;

pub struct EarlyZPass {
    pub framebuffer: Framebuffer,
    pub depth_clear_value: ClearValue,
    #[allow(dead_code)]
    pub depth_image_view: ImageView, // must survive for Framebuffer usage
    pub extent: Extent2D,
    pub render_pass: RenderPass,
}

impl EarlyZPass {
    pub fn new(
        device: &Device,
        depth_image: &ImageWrap,
        reversed_depth_buffer: bool)
        -> Result<EarlyZPass>
    {
        let render_pass = {
            use dacite::core::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout,
                               SubpassDescription, SubpassDescriptionFlags,
                               PipelineBindPoint, SubpassIndex, SubpassDependency,
                               PipelineStageFlags, AccessFlags, DependencyFlags,
                               RenderPassCreateFlags, RenderPassCreateInfo,
                               AttachmentReference, AttachmentIndex};

            let depth_attachment_description = depth_image.get_attachment_description(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
                ImageLayout::DepthStencilAttachmentOptimal,
                ImageLayout::DepthStencilAttachmentOptimal
            );

            let depth_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(0),
                layout: ImageLayout::DepthStencilAttachmentOptimal
            };

            let subpass = SubpassDescription {
                flags: SubpassDescriptionFlags::empty(),
                pipeline_bind_point: PipelineBindPoint::Graphics,
                input_attachments: vec![],
                color_attachments: vec![],
                resolve_attachments: vec![],
                depth_stencil_attachment: Some(depth_attachment_reference),
                preserve_attachments: vec![],
            };

            // We must write the depth buffer before the next RenderPass reads it
            let earlyz_to_shading = SubpassDependency {
                src_subpass: SubpassIndex::Index(0), // us
                dst_subpass: SubpassIndex::External, // next pass
                src_stage_mask: PipelineStageFlags::LATE_FRAGMENT_TESTS,
                dst_stage_mask: PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            let create_info = RenderPassCreateInfo {
                flags: RenderPassCreateFlags::empty(),
                attachments: vec![
                    depth_attachment_description,
                ],
                subpasses: vec![subpass],
                dependencies: vec![earlyz_to_shading],
                chain: None,
            };

            device.create_render_pass(&create_info, None)?
        };

        let (depth_image_view, framebuffer, extent) =
            build(device, render_pass.clone(), depth_image)?;

        Ok(EarlyZPass {
            framebuffer: framebuffer,
            depth_clear_value: depth_image.get_clear_value(reversed_depth_buffer),
            depth_image_view: depth_image_view,
            extent: extent,
            render_pass: render_pass,
        })
    }

    pub fn rebuild(&mut self, device: &Device, depth_image: &ImageWrap)
                   -> Result<()>
    {
        let (depth_image_view, framebuffer, extent) =
            build(device, self.render_pass.clone(), depth_image)?;

        self.framebuffer = framebuffer;
        self.depth_image_view = depth_image_view;
        self.extent = extent;

        Ok(())
    }

    pub fn record_entry(&self,
                        command_buffer: CommandBuffer)
                        -> Result<()>
    {
        use dacite::core::{Rect2D, Offset2D,
                           SubpassContents, RenderPassBeginInfo};

        let begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            framebuffer: self.framebuffer.clone(),
            render_area: Rect2D::new(Offset2D::zero(), self.extent),
            clear_values:  vec![
                self.depth_clear_value
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

fn build(device: &Device, render_pass: RenderPass, depth_image: &ImageWrap)
    -> Result<(ImageView, Framebuffer, Extent2D)>
{
    let depth_image_view = depth_image.get_image_view(device)?;

    let extent = Extent2D {
        width: depth_image.extent.width,
        height: depth_image.extent.height
    };

    let framebuffer = {
        use dacite::core::{FramebufferCreateInfo, FramebufferCreateFlags};

        let create_info = FramebufferCreateInfo {
            flags: FramebufferCreateFlags::empty(),
            render_pass: render_pass,
            attachments: vec![
                depth_image_view.clone()
            ],
            width: extent.width,
            height: extent.height,
            layers: 1,
            chain: None,
        };
        device.create_framebuffer(&create_info, None)?
    };

    Ok((depth_image_view, framebuffer, extent))
}
