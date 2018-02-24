
use errors::*;
use dacite::core::{Image, Format, ImageUsageFlags, Device, ImageView,
                   Extent3D, ImageLayout, ImageTiling, AccessFlags,
                   ImageSubresourceRange, Buffer, PipelineStageFlags,
                   ComponentMapping, AttachmentDescription,
                   AttachmentLoadOp, AttachmentStoreOp, ClearValue,
                   CommandBuffer};
use super::memory::{Memory, Block, Lifetime};
use super::commander::Commander;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageWrapType {
    Depth,
    Standard,
    //StandardMip(u32),
    Cubemap,
    //CubemapMip(u32),
    //Array(u32),
    //ArrayMip(u32,u32)),
    Swapchain
}

/// Encapsulated handling of images. Current code is limited to:
///   2D images, single MIP, single array layer, sharing mode exclusive
///   single sample count
#[derive(Debug, Clone)]
pub struct ImageWrap {
    pub image: Image,
    pub format: Format,
    pub extent: Extent3D,
    pub image_wrap_type: ImageWrapType,
    pub tiling: ImageTiling,
    pub usage: ImageUsageFlags,
    pub size: u64,
    pub block: Option<Block>,
    pub swizzle: ComponentMapping,
}

impl ImageWrap {
    pub fn new(
        device: &Device,
        memory: &mut Memory,
        format: Format,
        swizzle: ComponentMapping,
        extent: Extent3D,
        image_wrap_type: ImageWrapType,
        initial_layout: ImageLayout,
        mut tiling: ImageTiling,
        mut usage: ImageUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<ImageWrap>
    {
        use dacite::core::{ImageCreateInfo, ImageCreateFlags, ImageType,
                           SampleCountFlagBits, SharingMode, MemoryPropertyFlags};

        if image_wrap_type == ImageWrapType::Depth {
            usage = usage | ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
            tiling = ImageTiling::Optimal;
        }

        let image = {
            let create_info = ImageCreateInfo {
                // could set sparcity, mutable, cube-compat
                flags: match image_wrap_type {
                    ImageWrapType::Cubemap => ImageCreateFlags::CUBE_COMPATIBLE,
                    _ => ImageCreateFlags::empty(),
                },
                image_type: ImageType::Type2D,
                format: format,
                extent: extent,
                mip_levels: 1, // no LOD
                array_layers: match image_wrap_type {
                    ImageWrapType::Cubemap => 6,
                    _ => 1,
                },
                samples: SampleCountFlagBits::SampleCount1,
                tiling: tiling,
                usage: usage,
                sharing_mode: SharingMode::Exclusive,
                queue_family_indices: vec![], // ignored if not concurrent
                initial_layout: initial_layout,
                chain: None,
            };
            device.create_image(&create_info, None)?
        };

        let memory_requirements = image.get_memory_requirements();
        let block = memory.allocate_device_memory(
            device,
            &memory_requirements,
            MemoryPropertyFlags::DEVICE_LOCAL,
            lifetime,
            reason)?;

        image.bind_memory(block.memory.clone(), block.offset)?;

        Ok(ImageWrap {
            image: image,
            format: format,
            extent: extent,
            image_wrap_type: image_wrap_type,
            tiling: tiling,
            usage: usage,
            size: memory_requirements.size,
            block: Some(block),
            swizzle: swizzle
        })
    }

    pub fn get_image_view(&self, device: &Device) -> Result<ImageView>
    {
        use dacite::core::{ImageViewCreateInfo, ImageViewType,
                           ImageSubresourceRange, ImageAspectFlags,
                           OptionalMipLevels, OptionalArrayLayers};

        let create_info = ImageViewCreateInfo {
            flags: Default::default(),
            image: self.image.clone(),
            view_type: match self.image_wrap_type {
                ImageWrapType::Cubemap => ImageViewType::TypeCube,
                _ => ImageViewType::Type2D,
            },
            format: self.format,
            components: self.swizzle,
            subresource_range: ImageSubresourceRange {
                aspect_mask: if self.image_wrap_type == ImageWrapType::Depth {
                    ImageAspectFlags::DEPTH
                } else {
                    ImageAspectFlags::COLOR
                },
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: match self.image_wrap_type {
                    ImageWrapType::Cubemap => OptionalArrayLayers::ArrayLayers(6),
                    _ => OptionalArrayLayers::ArrayLayers(1),
                }
            },
            chain: None,
        };

        Ok(device.create_image_view(&create_info, None)?)
    }

    pub fn transition_layout_now(&mut self,
                                 device: &Device,
                                 src_layout: ImageLayout, dst_layout: ImageLayout,
                                 src_access: AccessFlags, dst_access: AccessFlags,
                                 src_stage: PipelineStageFlags, dst_stage: PipelineStageFlags,
                                 subresource_range: ImageSubresourceRange,
                                 commander: &Commander)
                                 -> Result<()>
    {
        use dacite::core::{CommandBufferBeginInfo, CommandBufferUsageFlags,
                           CommandBufferResetFlags,
                           Fence, FenceCreateInfo, FenceCreateFlags,
                           SubmitInfo, Timeout};

        commander.gfx_command_buffers[0].reset(CommandBufferResetFlags::RELEASE_RESOURCES)?;

        let command_buffer_begin_info = CommandBufferBeginInfo {
            flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            inheritance_info: None,
            chain: None
        };
        commander.gfx_command_buffers[0].begin(&command_buffer_begin_info)?;

        self.transition_layout(
            commander.gfx_command_buffers[0].clone(),
            src_layout, dst_layout,
            src_access, dst_access,
            src_stage, dst_stage,
            subresource_range)?;

        commander.gfx_command_buffers[0].end()?;

        let fence = {
            let create_info = FenceCreateInfo {
                flags: FenceCreateFlags::empty(),
                chain: None
            };
            device.create_fence(&create_info, None)?
        };

        let submit_info = SubmitInfo {
            wait_semaphores: vec![],
            wait_dst_stage_mask: vec![PipelineStageFlags::BOTTOM_OF_PIPE],
            command_buffers: vec![commander.gfx_command_buffers[0].clone()],
            signal_semaphores: vec![],
            chain: None
        };
        Fence::reset_fences(&[fence.clone()])?;
        commander.gfx_queue.submit( Some(&[submit_info]), Some(&fence) )?;
        Fence::wait_for_fences(&[fence], true, Timeout::Infinite)?;

        Ok(())
    }

    pub fn transition_layout(&mut self,
                             command_buffer: CommandBuffer,
                             src_layout: ImageLayout, dst_layout: ImageLayout,
                             src_access: AccessFlags, dst_access: AccessFlags,
                             src_stage: PipelineStageFlags, dst_stage: PipelineStageFlags,
                             subresource_range: ImageSubresourceRange)
                             -> Result<()>
    {
        use dacite::core::{ImageMemoryBarrier, QueueFamilyIndex,
                           DependencyFlags};

        let layout_transition_barrier = ImageMemoryBarrier {
            src_access_mask: src_access,
            dst_access_mask: dst_access,
            old_layout: src_layout,
            new_layout: dst_layout,
            src_queue_family_index: QueueFamilyIndex::Ignored,
            dst_queue_family_index: QueueFamilyIndex::Ignored,
            image: self.image.clone(),
            subresource_range: subresource_range,
            chain: None
        };

        command_buffer.pipeline_barrier(
            src_stage,
            dst_stage,
            DependencyFlags::empty(),
            None, //memory barriers
            None , //buffer memory barriers
            Some(&[layout_transition_barrier])); //image memory barriers

        Ok(())
    }

    // Currently this copies the entire buffer to an entire image;
    // we could improve it by coping regions/extents.
    pub fn copy_in_from_buffer(
        &mut self,
        device: &Device,
        commander: &Commander,
        buffer: &Buffer)
        -> Result<()>
    {
        use dacite::core::{CommandBufferBeginInfo, CommandBufferUsageFlags,
                           CommandBufferResetFlags,
                           ImageMemoryBarrier, QueueFamilyIndex,
                           PipelineStageFlags, DependencyFlags,
                           FenceCreateInfo, FenceCreateFlags,
                           SubmitInfo, Timeout, ImageAspectFlags, OptionalMipLevels,
                           OptionalArrayLayers, BufferImageCopy,
                           ImageSubresourceLayers, Offset3D};

        commander.xfr_command_buffer.reset(CommandBufferResetFlags::RELEASE_RESOURCES)?;

        let command_buffer_begin_info = CommandBufferBeginInfo {
            flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            inheritance_info: None,
            chain: None,
        };
        commander.xfr_command_buffer.begin(&command_buffer_begin_info)?;

        let image_barrier = ImageMemoryBarrier {
            src_access_mask: AccessFlags::empty(),
            dst_access_mask: AccessFlags::TRANSFER_WRITE,
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::TransferDstOptimal,
            src_queue_family_index: QueueFamilyIndex::Ignored,
            dst_queue_family_index: QueueFamilyIndex::Ignored,
            image: self.image.clone(),
            subresource_range: ImageSubresourceRange {
                aspect_mask: if self.image_wrap_type == ImageWrapType::Depth {
                    ImageAspectFlags::DEPTH
                } else {
                    ImageAspectFlags::COLOR
                },
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: match self.image_wrap_type {
                    ImageWrapType::Cubemap => OptionalArrayLayers::ArrayLayers(6),
                    _ => OptionalArrayLayers::ArrayLayers(1),
                }
            },
            chain: None,
        };
        commander.xfr_command_buffer.pipeline_barrier(
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            None,
            None,
            Some(&[image_barrier]));

        let buffer_copy_regions = BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0, // 0 means 'tightly packed' according to image_extent,
            buffer_image_height: 0, // 0 means 'tightly packed' according to image_extent,
            image_subresource: ImageSubresourceLayers {
                aspect_mask: if self.image_wrap_type == ImageWrapType::Depth {
                    ImageAspectFlags::DEPTH
                } else {
                    ImageAspectFlags::COLOR
                },
                mip_level: 0,
                base_array_layer: 0,
                layer_count: match self.image_wrap_type {
                    ImageWrapType::Cubemap => 6,
                    _ => 1,
                },
            },
            image_offset: Offset3D {
                x: 0,
                y: 0,
                z: 0
            },
            image_extent: self.extent,
        };
        commander.xfr_command_buffer.copy_buffer_to_image(
            buffer, //src_buffer
            &self.image, // dst_image
            ImageLayout::TransferDstOptimal, // dst_image_layout
            &[buffer_copy_regions], // regions
        );

        commander.xfr_command_buffer.end()?;

        let fence = {
            let create_info = FenceCreateInfo {
                flags: FenceCreateFlags::empty(),
                chain: None
            };
            device.create_fence(&create_info, None)?
        };
        // or fence.reset()?; if we take one as a parameter

        // submit the command buffer
        let submit_info = SubmitInfo {
            wait_semaphores: vec![],
            wait_dst_stage_mask: vec![PipelineStageFlags::TOP_OF_PIPE],
            command_buffers: vec![commander.xfr_command_buffer.clone()],
            signal_semaphores: vec![],
            chain: None,
        };
        commander.xfr_queue.submit(Some(&[submit_info]), Some(&fence))?;
        let _success = fence.wait_for(Timeout::Infinite)?;
        Ok(())
    }

    pub fn get_attachment_description(&self,
                                      load_op: AttachmentLoadOp,
                                      store_op: AttachmentStoreOp,
                                      initial_layout: ImageLayout,
                                      final_layout: ImageLayout)
                                      -> AttachmentDescription
    {
        use dacite::core::{AttachmentDescriptionFlags, SampleCountFlagBits};

        AttachmentDescription {
            flags: AttachmentDescriptionFlags::empty(),
            format: self.format,
            samples: SampleCountFlagBits::SampleCount1,
            load_op: load_op,
            store_op: store_op,
            stencil_load_op: AttachmentLoadOp::DontCare,
            stencil_store_op: AttachmentStoreOp::DontCare,
            initial_layout: initial_layout,
            final_layout: final_layout,
        }
    }

    pub fn get_clear_value(&self,
                           reversed_depth_buffer: bool)
                           -> ClearValue
    {
        use dacite::core::{ClearDepthStencilValue, ClearColorValue};

        match self.image_wrap_type {
            ImageWrapType::Depth => ClearValue::DepthStencil(
                ClearDepthStencilValue {
                    depth: if reversed_depth_buffer { 0.0 } else { 1.0 },
                    stencil: 0,
                }),
            _ => ClearValue::Color(
                ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0])),
        }
    }
}
