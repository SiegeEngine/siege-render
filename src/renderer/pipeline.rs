
use dacite::core::{Device, Viewport, Rect2D, RenderPass, ShaderModule,
                   Pipeline, PipelineLayout, PipelineLayoutCreateInfo,
                   DescriptorSetLayout, PrimitiveTopology, CullModeFlags, FrontFace,
                   GraphicsPipelineCreateInfo, PipelineCreateFlags,
                   PipelineShaderStageCreateInfo, PipelineShaderStageCreateFlags,
                   ShaderStageFlagBits,
                   PipelineInputAssemblyStateCreateInfo,
                   PipelineInputAssemblyStateCreateFlags,
                   PipelineVertexInputStateCreateInfo,
                   PipelineVertexInputStateCreateFlags,
                   PipelineViewportStateCreateInfo,
                   PipelineViewportStateCreateFlags,
                   PipelineRasterizationStateCreateInfo,
                   PipelineRasterizationStateCreateFlags,
                   PolygonMode,
                   PipelineMultisampleStateCreateInfo,
                   PipelineMultisampleStateCreateFlags,
                   SampleCountFlagBits,
                   PipelineColorBlendStateCreateInfo,
                   PipelineColorBlendStateCreateFlags,
                   LogicOp, BlendFactor, BlendOp,
                   PipelineColorBlendAttachmentState,
                   ColorComponentFlags,
                   CompareOp, StencilOp, StencilOpState,
                   PipelineDepthStencilStateCreateInfo,
                   PipelineDynamicStateCreateInfo, DynamicState,
                   PipelineLayoutCreateFlags,
                   SpecializationInfo, PushConstantRange};
use error::Error;
use super::{DepthHandling, BlendMode};

pub fn create(
    device: &Device,
    viewport: Viewport,
    scissors: Rect2D,
    reversed_depth_buffer: bool,
    render_pass: RenderPass,
    desc_set_layouts: Vec<DescriptorSetLayout>,
    vertex_shader: Option<ShaderModule>,
    vertex_spec_info: Option<SpecializationInfo>,
    fragment_shader: Option<ShaderModule>,
    fragment_spec_info: Option<SpecializationInfo>,
    vertex_type: Option<PipelineVertexInputStateCreateInfo>,
    topology: PrimitiveTopology,
    cull_mode: CullModeFlags,
    front_face: FrontFace,
    depth_handling: DepthHandling,
    blend: Vec<BlendMode>,
    push_constant_ranges: Vec<PushConstantRange>)
    -> Result<(PipelineLayout, Pipeline), Error>
{
    let layout = device.create_pipeline_layout(
        &PipelineLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            set_layouts: desc_set_layouts,
            push_constant_ranges: push_constant_ranges,
            chain: None,
        }, None)?;

    let blend_create = if blend.len() == 0 {
        None
    } else {
        Some(PipelineColorBlendStateCreateInfo {
            flags: PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: false,
            logic_op: LogicOp::Copy,
            attachments: blend.iter().map(
                |bm|
                PipelineColorBlendAttachmentState {
                    blend_enable: match bm {
                        &BlendMode::Off => false,
                        _ => true,
                    },
                    src_color_blend_factor: match bm {
                        &BlendMode::Add => BlendFactor::One,
                        &BlendMode::PreMultiplied => BlendFactor::One,
                        _ => BlendFactor::SrcAlpha,
                    },
                    dst_color_blend_factor: match bm {
                        &BlendMode::Add => BlendFactor::One,
                        _ => BlendFactor::OneMinusSrcAlpha,
                    },
                    color_blend_op: BlendOp::Add,
                    src_alpha_blend_factor: BlendFactor::One,
                    dst_alpha_blend_factor: BlendFactor::Zero,
                    alpha_blend_op: BlendOp::Add,
                    color_write_mask: ColorComponentFlags::R | ColorComponentFlags::G | ColorComponentFlags::B
                }).collect(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            chain: None,
        })
    };

    let mut create_info = GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::empty(),
        stages: vec![],
        vertex_input_state: match vertex_type {
            Some(vt) => vt,
            None => PipelineVertexInputStateCreateInfo {
                flags: PipelineVertexInputStateCreateFlags::empty(),
                vertex_binding_descriptions: vec![],
                vertex_attribute_descriptions: vec![],
                chain: None,
            }
        },
        input_assembly_state: PipelineInputAssemblyStateCreateInfo {
            flags: PipelineInputAssemblyStateCreateFlags::empty(),
            topology: topology,
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
            cull_mode: cull_mode,
            front_face: front_face,
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
        depth_stencil_state: match depth_handling {
            DepthHandling::None => None,
            DepthHandling::Some(test,write) => Some(PipelineDepthStencilStateCreateInfo {
                flags: Default::default(),
                depth_test_enable: test,
                depth_write_enable: write,
                depth_compare_op: if reversed_depth_buffer {
                    CompareOp::GreaterOrEqual
                } else {
                    CompareOp::LessOrEqual
                },
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
                front: StencilOpState {
                    fail_op: StencilOp::Keep,
                    pass_op: StencilOp::Keep,
                    depth_fail_op: StencilOp::Keep,
                    compare_op: CompareOp::Always,
                    compare_mask: 0,
                    write_mask: 0,
                    reference: 0,
                },
                back: StencilOpState {
                    fail_op: StencilOp::Keep,
                    pass_op: StencilOp::Keep,
                    depth_fail_op: StencilOp::Keep,
                    compare_op: CompareOp::Always,
                    compare_mask: 0,
                    write_mask: 0,
                    reference: 0,
                },
                min_depth_bounds: if reversed_depth_buffer { 1.0 } else { 0.0 },
                max_depth_bounds: if reversed_depth_buffer { 0.0 } else { 1.0 },
                chain: None,
            })
        },
        color_blend_state: blend_create,
        dynamic_state: Some(PipelineDynamicStateCreateInfo {
            flags: Default::default(),
            dynamic_states: vec![DynamicState::Viewport, DynamicState::Scissor],
            chain: None,
        }),
        layout: layout.clone(),
        render_pass: render_pass,
        subpass: 0,
        base_pipeline: None,
        base_pipeline_index: None,
        chain: None,
    };

    if let Some(vs) = vertex_shader {
        create_info.stages.push(
            PipelineShaderStageCreateInfo {
                flags: PipelineShaderStageCreateFlags::empty(),
                stage: ShaderStageFlagBits::Vertex,
                module: vs,
                name: "main".to_owned(),
                specialization_info: vertex_spec_info,
                chain: None,
            }
        );
    }

    if let Some(fs) = fragment_shader {
        create_info.stages.push(
            PipelineShaderStageCreateInfo {
                flags: PipelineShaderStageCreateFlags::empty(),
                stage: ShaderStageFlagBits::Fragment,
                module: fs,
                name: "main".to_owned(),
                specialization_info: fragment_spec_info,
                chain: None,
            }
        );
    }

    let create_infos = vec![create_info];
    let pipelines = device.create_graphics_pipelines(None, &create_infos, None)
        .map_err(|(e, _)| e)?;
    Ok((layout, pipelines[0].clone()))
}
