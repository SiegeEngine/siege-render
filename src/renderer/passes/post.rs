
use dacite::core::{Device, RenderPass, Framebuffer, Extent2D, ImageView,
                   CommandBuffer};
use crate::error::Error;
use crate::renderer::image_wrap::ImageWrap;
use crate::renderer::swapchain_data::SwapchainData;

pub struct PostPass {
    pub framebuffers: Vec<Framebuffer>,
    pub swapchain_image_views: Vec<ImageView>,
    pub shading_image_view: ImageView,
    pub extent: Extent2D,
    pub render_pass: RenderPass,
}

impl PostPass {
    pub fn new(
        device: &Device,
        shading_image: &ImageWrap,
        swapchain_data: &SwapchainData)
        -> Result<PostPass, Error>
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
                AttachmentStoreOp::DontCare,
                ImageLayout::ShaderReadOnlyOptimal,
                ImageLayout::ShaderReadOnlyOptimal,
            );

            let shading_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(0),
                layout: ImageLayout::ShaderReadOnlyOptimal
            };

            let swapchain_attachment_description = swapchain_data.images[0].get_attachment_description(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );

            let swapchain_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(1),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let subpass = SubpassDescription {
                flags: SubpassDescriptionFlags::empty(),
                pipeline_bind_point: PipelineBindPoint::Graphics,
                input_attachments: vec![shading_attachment_reference],
                color_attachments: vec![swapchain_attachment_reference],
                resolve_attachments: vec![],
                depth_stencil_attachment: None,
                preserve_attachments: vec![],
            };

            // We must have written the shading image before this pass reads it
            let blur_v_to_post = SubpassDependency {
                src_subpass: SubpassIndex::External, // bloom_v pass
                dst_subpass: SubpassIndex::Index(0), // us
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            // We must have written the swapchain before ui blends into swapchain
            let post_to_ui = SubpassDependency {
                src_subpass: SubpassIndex::Index(0), // us
                dst_subpass: SubpassIndex::External, // ui pass
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            let create_info = RenderPassCreateInfo {
                flags: RenderPassCreateFlags::empty(),
                attachments: vec![
                    shading_attachment_description, // 0
                    swapchain_attachment_description // 1
                ],
                subpasses: vec![subpass],
                dependencies: vec![
                    blur_v_to_post,
                    post_to_ui
                ],
                chain: None,
            };

            device.create_render_pass(&create_info, None)?
        };

        let (shading_image_view, swapchain_image_views, framebuffers, extent) =
            build(device, render_pass.clone(), shading_image, swapchain_data)?;

        Ok(PostPass {
            framebuffers: framebuffers,
            swapchain_image_views: swapchain_image_views,
            shading_image_view: shading_image_view,
            extent: extent,
            render_pass: render_pass,
        })
    }

    pub fn rebuild(&mut self, device: &Device,
                   shading_image: &ImageWrap,
                   swapchain_data: &SwapchainData)
                   -> Result<(), Error>
    {
        let (shading_image_view, swapchain_image_views, framebuffers, extent) =
            build(device, self.render_pass.clone(), shading_image, swapchain_data)?;

        self.framebuffers = framebuffers;
        self.shading_image_view = shading_image_view;
        self.swapchain_image_views = swapchain_image_views;
        self.extent = extent;

        Ok(())
    }

    pub fn record_entry(&self, command_buffer: CommandBuffer,
                        present_index: usize)
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
                ClearValue::Color(ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])), // unused
                ClearValue::Color(ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])),
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

fn build(device: &Device, render_pass: RenderPass, shading_image: &ImageWrap,
         swapchain_data: &SwapchainData)
    -> Result<(ImageView, Vec<ImageView>, Vec<Framebuffer>, Extent2D), Error>
{
    let extent = swapchain_data.extent;

    let shading_image_view = shading_image.get_image_view(device)?;

    let mut image_views = Vec::new();
    let mut framebuffers = Vec::new();

    for image in &swapchain_data.images {
        use dacite::core::{FramebufferCreateInfo, FramebufferCreateFlags};

        let image_view = image.get_image_view(device)?;

        let create_info = FramebufferCreateInfo {
            flags: FramebufferCreateFlags::empty(),
            render_pass: render_pass.clone(),
            attachments: vec![
                shading_image_view.clone(),
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

    Ok((shading_image_view, image_views, framebuffers, extent))
}
