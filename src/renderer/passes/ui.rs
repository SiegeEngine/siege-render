
use dacite::core::{Device, RenderPass, Framebuffer, Extent2D, ImageView,
                   CommandBuffer};
use errors::*;
use renderer::swapchain_data::SwapchainData;

pub struct UiPass {
    pub framebuffers: Vec<Framebuffer>,
    pub swapchain_image_views: Vec<ImageView>,
    pub extent: Extent2D,
    pub render_pass: RenderPass,
}

impl UiPass {
    pub fn new(
        device: &Device,
        swapchain_data: &SwapchainData)
        -> Result<UiPass>
    {
        let render_pass = {
            use dacite::core::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout,
                               SubpassDescription, SubpassDescriptionFlags,
                               PipelineBindPoint, SubpassIndex, SubpassDependency,
                               PipelineStageFlags, AccessFlags, DependencyFlags,
                               RenderPassCreateFlags, RenderPassCreateInfo,
                               AttachmentReference, AttachmentIndex};

            let swapchain_attachment_description = swapchain_data.images[0].get_attachment_description(
                AttachmentLoadOp::Load,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );

            let swapchain_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(0),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let subpass = SubpassDescription {
                flags: SubpassDescriptionFlags::empty(),
                pipeline_bind_point: PipelineBindPoint::Graphics,
                input_attachments: vec![],
                color_attachments: vec![swapchain_attachment_reference],
                resolve_attachments: vec![],
                depth_stencil_attachment: None,
                preserve_attachments: vec![],
            };

            // We must have written the shading buffer before this pass reads it
            let post_to_ui = SubpassDependency {
                src_subpass: SubpassIndex::External, // post pass
                dst_subpass: SubpassIndex::Index(0), // us
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            let create_info = RenderPassCreateInfo {
                flags: RenderPassCreateFlags::empty(),
                attachments: vec![
                    swapchain_attachment_description,
                ],
                subpasses: vec![subpass],
                dependencies: vec![
                    post_to_ui,
                ],
                chain: None,
            };

            device.create_render_pass(&create_info, None)?
        };

        let (swapchain_image_views, framebuffers, extent) =
            build(device, render_pass.clone(), swapchain_data)?;

        Ok(UiPass {
            framebuffers: framebuffers,
            swapchain_image_views: swapchain_image_views,
            extent: extent,
            render_pass: render_pass,
        })
    }

    pub fn rebuild(&mut self, device: &Device,
                   swapchain_data: &SwapchainData)
                   -> Result<()>
    {
        let (swapchain_image_views, framebuffers, extent) =
            build(device, self.render_pass.clone(), swapchain_data)?;

        self.framebuffers = framebuffers;
        self.swapchain_image_views = swapchain_image_views;
        self.extent = extent;

        Ok(())
    }

    pub fn record_entry(&self,
                        command_buffer: CommandBuffer,
                        present_index: usize)
                        -> Result<()>
    {
        use dacite::core::{Rect2D, Offset2D,
                           SubpassContents, RenderPassBeginInfo,
                           ClearValue, ClearColorValue};

        let begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            framebuffer: self.framebuffers[present_index].clone(),
            render_area: Rect2D::new(Offset2D::zero(), self.extent),
            clear_values:  vec![
                ClearValue::Color(ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])), // unused
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

fn build(device: &Device, render_pass: RenderPass, swapchain_data: &SwapchainData)
    -> Result<(Vec<ImageView>, Vec<Framebuffer>, Extent2D)>
{
    let extent = swapchain_data.extent;

    let mut image_views = Vec::new();
    let mut framebuffers = Vec::new();

    for image in &swapchain_data.images {
        use dacite::core::{FramebufferCreateInfo, FramebufferCreateFlags};

        let image_view = image.get_image_view(device)?;

        let create_info = FramebufferCreateInfo {
            flags: FramebufferCreateFlags::empty(),
            render_pass: render_pass.clone(),
            attachments: vec![
                image_view.clone(),
            ],
            width: extent.width,
            height: extent.height,
            layers: 1,
            chain: None,
        };
        let framebuffer = device.create_framebuffer(&create_info, None)?;

        image_views.push(image_view);
        framebuffers.push(framebuffer);
    };

    Ok((image_views, framebuffers, extent))
}
