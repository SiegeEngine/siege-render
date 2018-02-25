
use dacite::core::{Device, DescriptorPool, DescriptorSet, DescriptorSetLayout,
                   DescriptorSetLayoutBinding, Sampler, ImageView, ImageLayout,
                   DescriptorType, CommandBuffer, RenderPass, Viewport, Rect2D,
                   PipelineBindPoint, Pipeline, PipelineLayout, PrimitiveTopology,
                   CullModeFlags, FrontFace};
use errors::*;
use super::target_data::TargetData;
use super::resource_manager::ResourceManager;
use super::{DepthHandling, BlendMode};

pub struct PostGfx {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    #[allow(dead_code)] // this must remain alive
    desc_layout: DescriptorSetLayout,
    shading_image_view: ImageView,
    sampler: Sampler,
}

impl PostGfx {
    pub fn new(device: &Device,
               descriptor_pool: DescriptorPool,
               target_data: &TargetData,
               resource_manager: &mut ResourceManager,
               render_pass: RenderPass,
               viewport: Viewport,
               scissors: Rect2D)
               -> Result<PostGfx>
    {
        let sampler = {
            use dacite::core::{SamplerCreateInfo, SamplerMipmapMode, SamplerAddressMode,
                               BorderColor, Filter, CompareOp};

            device.create_sampler(&SamplerCreateInfo {
                flags: Default::default(),
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode_u: SamplerAddressMode::ClampToEdge,
                address_mode_v: SamplerAddressMode::ClampToEdge,
                address_mode_w: SamplerAddressMode::ClampToEdge,
                mip_lod_bias: 0.0,
                anisotropy_enable: false,
                max_anisotropy: 1.0,
                compare_enable: false,
                compare_op: CompareOp::Never,
                min_lod: 0.0,
                max_lod: 1.0,
                border_color: BorderColor::FloatOpaqueWhite,
                unnormalized_coordinates: false,
                chain: None
            }, None)?
        };

        let shading_image_view = target_data.shading_image.
            get_image_view(device)?;

        let desc_bindings = {
            use dacite::core::{DescriptorType, ShaderStageFlags};
            vec![
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: vec![],
                },
            ]
        };

        let desc_layout = {
            use dacite::core::DescriptorSetLayoutCreateInfo;

            let create_info = DescriptorSetLayoutCreateInfo {
                flags: Default::default(),
                bindings: desc_bindings.clone(),
                chain: None,
            };
            device.create_descriptor_set_layout(&create_info, None)?
        };

        let descriptor_set = {
            use dacite::core::DescriptorSetAllocateInfo;

            let alloc_info = DescriptorSetAllocateInfo {
                descriptor_pool: descriptor_pool.clone(),
                set_layouts: vec![desc_layout.clone()],
                chain: None,
            };

            let mut descriptor_sets = DescriptorPool::allocate_descriptor_sets(&alloc_info)?;
            descriptor_sets.pop().unwrap()
        };

        let vertex_shader = resource_manager.load_shader(&device, "post2.vert")?;
        let fragment_shader = resource_manager.load_shader(&device, "post2.frag")?;
        let (pipeline_layout, pipeline) =
            super::pipeline::create(
                device, viewport, scissors,
                true, // reversed depth buffer irrelevant for post
                render_pass, vec![desc_layout.clone()],
                Some(vertex_shader), Some(fragment_shader),
                None,
                PrimitiveTopology::TriangleList,
                CullModeFlags::NONE, FrontFace::Clockwise,
                DepthHandling::None,
                BlendMode::None)?;

        let mut post_gfx = PostGfx {
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
            descriptor_set: descriptor_set,
            desc_layout: desc_layout,
            shading_image_view: shading_image_view,
            sampler: sampler,
        };

        post_gfx.write();

        Ok(post_gfx)
    }

    pub fn rebuild(&mut self, device: &Device, target_data: &TargetData)
        -> Result<()>
    {
        self.shading_image_view = target_data.shading_image.
            get_image_view(device)?;

        self.write();

        Ok(())
    }

    fn write(&mut self)
    {
        use dacite::core::{WriteDescriptorSet, WriteDescriptorSetElements,
                           DescriptorImageInfo};

        DescriptorSet::update(
            Some(&[
                WriteDescriptorSet {
                    dst_set: self.descriptor_set.clone(),
                    dst_binding: 0,
                    dst_array_element: 0, // only have 1 element
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    elements: WriteDescriptorSetElements::ImageInfo(
                        vec![
                            DescriptorImageInfo {
                                sampler: Some(self.sampler.clone()),
                                image_view: Some(self.shading_image_view.clone()),
                                image_layout: ImageLayout::ShaderReadOnlyOptimal,
                            }
                        ]
                    ),
                    chain: None,
                },
            ]),
            None
        );
    }

    pub fn record(&self, command_buffer: CommandBuffer)
    {
        // Bind our pipeline
        command_buffer.bind_pipeline(PipelineBindPoint::Graphics, &self.pipeline);

        // FIXME: bind shading_image as a texture desc set
        command_buffer.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            &self.pipeline_layout,
            0, // starting with first set
            &[self.descriptor_set.clone()],
            None,
        );

        command_buffer.draw(3, 1, 0, 0);
    }
}
