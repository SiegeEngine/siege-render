
use dacite::core::{Device, DescriptorPool, DescriptorSet, DescriptorSetLayout,
                   DescriptorSetLayoutBinding, Sampler, ImageView, ImageLayout,
                   DescriptorType, CommandBuffer, RenderPass, Viewport, Rect2D,
                   PipelineBindPoint, Pipeline, PipelineLayout};
use errors::*;
use super::target_data::TargetData;
use super::resource_manager::ResourceManager;

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

        let pipeline_layout = build_pipeline_layout(
            device,
            desc_layout.clone()
        )?;

        let pipeline = build_pipeline(
            device,
            resource_manager,
            render_pass,
            viewport,
            scissors,
            pipeline_layout.clone()
        )?;

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
                  -> Result<()>
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

        Ok(())
    }
}

fn build_pipeline_layout(
    device: &Device,
    descriptor_set_layout: DescriptorSetLayout)
    -> Result<PipelineLayout>
{
    use dacite::core::{PipelineLayoutCreateInfo, PipelineLayoutCreateFlags};

    let create_info = PipelineLayoutCreateInfo {
        flags: PipelineLayoutCreateFlags::empty(),
        set_layouts: vec![descriptor_set_layout],
        push_constant_ranges: vec![],
        chain: None,
    };
    Ok(device.create_pipeline_layout(&create_info, None)?)
}

fn build_pipeline(
    device: &Device,
    resource_manager: &mut ResourceManager,
    render_pass: RenderPass,
    viewport: Viewport,
    scissors: Rect2D,
    pipeline_layout: PipelineLayout)
    -> Result<Pipeline>
{
    use dacite::core::{GraphicsPipelineCreateInfo, PipelineCreateFlags,
                       PipelineShaderStageCreateInfo, PipelineShaderStageCreateFlags,
                       ShaderStageFlagBits,
                       PipelineInputAssemblyStateCreateInfo,
                       PipelineInputAssemblyStateCreateFlags,
                       PrimitiveTopology,
                       PipelineViewportStateCreateInfo,
                       PipelineViewportStateCreateFlags,
                       PipelineRasterizationStateCreateInfo,
                       PipelineRasterizationStateCreateFlags,
                       PolygonMode, CullModeFlags, FrontFace,
                       PipelineMultisampleStateCreateInfo,
                       PipelineMultisampleStateCreateFlags,
                       SampleCountFlagBits,
                       PipelineColorBlendStateCreateInfo,
                       PipelineColorBlendStateCreateFlags,
                       LogicOp, BlendFactor, BlendOp,
                       PipelineColorBlendAttachmentState,
                       ColorComponentFlags,
                       PipelineDynamicStateCreateInfo, DynamicState,
                       PipelineVertexInputStateCreateInfo,
                       PipelineVertexInputStateCreateFlags};

    let vertex_shader = resource_manager.load_shader(&device, "post2.vert")?;

    let fragment_shader = resource_manager.load_shader(&device, "post2.frag")?;

    let create_infos = vec![GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::empty(),
        stages: vec![
            // A vertex shader is required by vulkan.
            PipelineShaderStageCreateInfo {
                flags: PipelineShaderStageCreateFlags::empty(),
                stage: ShaderStageFlagBits::Vertex,
                module: vertex_shader,
                name: "main".to_owned(),
                specialization_info: None,
                chain: None,
            },
            PipelineShaderStageCreateInfo {
                flags: PipelineShaderStageCreateFlags::empty(),
                stage: ShaderStageFlagBits::Fragment,
                module: fragment_shader,
                name: "main".to_owned(),
                specialization_info: None,
                chain: None,
            },
        ],
        // we have NO vertex input, we compute from gl_VertexIndex within the shader
        vertex_input_state: PipelineVertexInputStateCreateInfo {
            flags: PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_descriptions: vec![],
            vertex_attribute_descriptions: vec![],
            chain: None,
        },
        input_assembly_state: PipelineInputAssemblyStateCreateInfo {
            flags: PipelineInputAssemblyStateCreateFlags::empty(),
            topology: PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            chain: None,
        },
        tessellation_state: None,
        viewport_state: Some(PipelineViewportStateCreateInfo {
            flags: PipelineViewportStateCreateFlags::empty(),
            viewports: vec![viewport],
            scissors: vec![scissors],
            chain: None,
        }),
        rasterization_state: PipelineRasterizationStateCreateInfo {
            flags: PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: PolygonMode::Fill,
            cull_mode: CullModeFlags::NONE, // TEMP
            front_face: FrontFace::Clockwise,
            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
            chain: None,
        },
        multisample_state: Some(PipelineMultisampleStateCreateInfo {
            flags: PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: SampleCountFlagBits::SampleCount1,
            sample_shading_enable: false,
            min_sample_shading: 0.0,
            sample_mask: vec![],
            alpha_to_coverage_enable: false,
            alpha_to_one_enable: false,
            chain: None,
        }),
        depth_stencil_state: None,
        color_blend_state: Some(PipelineColorBlendStateCreateInfo {
            flags: PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: false,
            logic_op: LogicOp::Clear,
            attachments: vec![PipelineColorBlendAttachmentState {
                blend_enable: false, // no blending to final image
                // outputColor = colorBlendOp(
                //      srcColor * srcColorBlendFactor, dstColor * dstColorBlendFactor);
                src_color_blend_factor: BlendFactor::SrcAlpha,
                dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                color_blend_op: BlendOp::Add,
                src_alpha_blend_factor: BlendFactor::One,
                dst_alpha_blend_factor: BlendFactor::Zero,
                alpha_blend_op: BlendOp::Add,
                color_write_mask: ColorComponentFlags::R | ColorComponentFlags::G | ColorComponentFlags::B
            }],
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            chain: None,
        }),
        dynamic_state: Some(PipelineDynamicStateCreateInfo {
            flags: Default::default(),
            dynamic_states: vec![DynamicState::Viewport, DynamicState::Scissor],
            chain: None,
        }),
        layout: pipeline_layout,
        render_pass: render_pass,
        subpass: 0,
        base_pipeline: None,
        base_pipeline_index: None,
        chain: None,
    }];

    let pipelines = device.create_graphics_pipelines(None, &create_infos, None)
        .map_err(|(e, _)| e)?;

    Ok(pipelines[0].clone())
}
