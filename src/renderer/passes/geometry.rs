
use dacite::core::{Device, RenderPass, Framebuffer, Extent2D, ImageView,
                   CommandBuffer, ClearValue, ClearColorValue};
use crate::error::Error;
use crate::renderer::image_wrap::ImageWrap;

pub struct GeometryPass {
    pub framebuffer: Framebuffer,
    pub material_image_view: ImageView,
    pub normals_image_view: ImageView,
    pub diffuse_image_view: ImageView,
    pub depth_clear_value: ClearValue,
    pub depth_image_view: ImageView,
    pub extent: Extent2D,
    pub render_pass: RenderPass,
}

impl GeometryPass {
    pub fn new(
        device: &Device,
        depth_image: &ImageWrap,
        diffuse_image: &ImageWrap,
        normals_image: &ImageWrap,
        material_image: &ImageWrap,
        reversed_depth_buffer: bool)
        -> Result<GeometryPass, Error>
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

            let diffuse_attachment_description = diffuse_image.get_attachment_description(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );
            let diffuse_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(1),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let normals_attachment_description = normals_image.get_attachment_description(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );
            let normals_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(2),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let material_attachment_description = material_image.get_attachment_description(
                AttachmentLoadOp::Clear,
                AttachmentStoreOp::Store,
                ImageLayout::ColorAttachmentOptimal,
                ImageLayout::ColorAttachmentOptimal,
            );
            let material_attachment_reference = AttachmentReference {
                attachment: AttachmentIndex::Index(3),
                layout: ImageLayout::ColorAttachmentOptimal
            };

            let subpass = SubpassDescription {
                flags: SubpassDescriptionFlags::empty(),
                pipeline_bind_point: PipelineBindPoint::Graphics,
                input_attachments: vec![],
                color_attachments: vec![diffuse_attachment_reference,
                                        normals_attachment_reference,
                                        material_attachment_reference],
                resolve_attachments: vec![],
                depth_stencil_attachment: Some(depth_attachment_reference),
                preserve_attachments: vec![],
            };

            // We must write the depth buffer before the next RenderPass reads it
            let geometry_to_shading_1 = SubpassDependency {
                src_subpass: SubpassIndex::Index(0), // us
                dst_subpass: SubpassIndex::External, // next pass
                src_stage_mask: PipelineStageFlags::LATE_FRAGMENT_TESTS,
                dst_stage_mask: PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };
            // We must write the color attachments before the next RenderPass
            // reads them
            let geometry_to_shading_2 = SubpassDependency {
                src_subpass: SubpassIndex::Index(0), // us
                dst_subpass: SubpassIndex::External, // next pass
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ,
                dependency_flags:  DependencyFlags::BY_REGION,
            };

            let create_info = RenderPassCreateInfo {
                flags: RenderPassCreateFlags::empty(),
                attachments: vec![
                    depth_attachment_description,
                    diffuse_attachment_description,
                    normals_attachment_description,
                    material_attachment_description
                ],
                subpasses: vec![subpass],
                dependencies: vec![
                    geometry_to_shading_1,
                    geometry_to_shading_2],
                chain: None,
            };

            device.create_render_pass(&create_info, None)?
        };

        let (depth_image_view, diffuse_image_view, normals_image_view,
             material_image_view, framebuffer, extent) = build(
            device, render_pass.clone(), depth_image, diffuse_image,
            normals_image, material_image)?;

        Ok(GeometryPass {
            framebuffer: framebuffer,
            material_image_view: material_image_view,
            normals_image_view: normals_image_view,
            diffuse_image_view: diffuse_image_view,
            depth_clear_value: depth_image.get_clear_value(reversed_depth_buffer),
            depth_image_view: depth_image_view,
            extent: extent,
            render_pass: render_pass,
        })
    }

    pub fn rebuild(&mut self, device: &Device, depth_image: &ImageWrap,
                   diffuse_image: &ImageWrap, normals_image: &ImageWrap,
                   material_image: &ImageWrap)
                   -> Result<(), Error>
    {
        let (depth_image_view, diffuse_image_view, normals_image_view,
             material_image_view, framebuffer, extent) = build(
            device, self.render_pass.clone(), depth_image, diffuse_image,
            normals_image, material_image)?;

        self.framebuffer = framebuffer;
        self.depth_image_view = depth_image_view;
        self.diffuse_image_view = diffuse_image_view;
        self.normals_image_view = normals_image_view;
        self.material_image_view = material_image_view;
        self.extent = extent;

        Ok(())
    }

    pub fn record_entry(&self, command_buffer: CommandBuffer)
    {
        use dacite::core::{Rect2D, Offset2D,
                           SubpassContents, RenderPassBeginInfo};

        let begin_info = RenderPassBeginInfo {
            render_pass: self.render_pass.clone(),
            framebuffer: self.framebuffer.clone(),
            render_area: Rect2D::new(Offset2D::zero(), self.extent),
            clear_values:  vec![
                self.depth_clear_value,
                ClearValue::Color(ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])),
                ClearValue::Color(ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])),
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

fn build(device: &Device, render_pass: RenderPass, depth_image: &ImageWrap,
         diffuse_image: &ImageWrap, normals_image: &ImageWrap,
         material_image: &ImageWrap)
    -> Result<(ImageView, ImageView, ImageView, ImageView, Framebuffer, Extent2D), Error>
{
    let depth_image_view = depth_image.get_image_view(device)?;
    let diffuse_image_view = diffuse_image.get_image_view(device)?;
    let normals_image_view = normals_image.get_image_view(device)?;
    let material_image_view = material_image.get_image_view(device)?;

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
                depth_image_view.clone(),
                diffuse_image_view.clone(),
                normals_image_view.clone(),
                material_image_view.clone()
            ],
            width: extent.width,
            height: extent.height,
            layers: 1,
            chain: None,
        };
        device.create_framebuffer(&create_info, None)?
    };

    Ok((depth_image_view, diffuse_image_view, normals_image_view,
        material_image_view, framebuffer, extent))
}
