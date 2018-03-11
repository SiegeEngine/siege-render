
use std::collections::HashMap;
use dacite::core::{Device, DescriptorPool, DescriptorSet, DescriptorSetLayout,
                   DescriptorSetLayoutBinding, Sampler, ImageView, ImageLayout,
                   DescriptorType, CommandBuffer, RenderPass, Viewport, Rect2D,
                   PipelineBindPoint, Pipeline, PipelineLayout, PrimitiveTopology,
                   CullModeFlags, FrontFace, ShaderModuleCreateFlags,
                   ShaderModuleCreateInfo, ShaderModule};
use siege_math::Vec3;
use errors::*;
use super::target_data::TargetData;
use super::{DepthHandling, BlendMode};

pub struct DLight {
    pub direction: Vec3<f32>,
    pub irradiance: Vec3<f32>,
}

pub struct ShadeGfx {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    #[allow(dead_code)] // this must remain alive
    desc_layout: DescriptorSetLayout,
    material_image_view: ImageView,
    normals_image_view: ImageView,
    diffuse_image_view: ImageView,
    sampler: Sampler,
    directional_lights: HashMap<u32, DLight>,
    next_dlight_token: u32,
}

impl ShadeGfx {
    pub fn new(device: &Device,
               descriptor_pool: DescriptorPool,
               target_data: &TargetData,
               render_pass: RenderPass,
               viewport: Viewport,
               scissors: Rect2D,
               params_layout: DescriptorSetLayout,
               reversed_depth_buffer: bool)
               -> Result<ShadeGfx>
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

        let diffuse_image_view = target_data.diffuse_image.get_image_view(device)?;
        let normals_image_view = target_data.normals_image.get_image_view(device)?;
        let material_image_view = target_data.material_image.get_image_view(device)?;

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
                DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: vec![],
                },
                DescriptorSetLayoutBinding {
                    binding: 2,
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
                set_layouts: vec![desc_layout.clone(),],
                chain: None,
            };

            let mut descriptor_sets = DescriptorPool::allocate_descriptor_sets(&alloc_info)?;
            descriptor_sets.pop().unwrap()
        };

        let vertex_shader = vertex_shader(device)?;

        let fragment_shader = fragment_shader(device)?;

        let (pipeline_layout, pipeline) =
            super::pipeline::create(
                device, viewport, scissors,
                reversed_depth_buffer,
                render_pass, vec![desc_layout.clone(),
                                  params_layout],
                Some(vertex_shader), Some(fragment_shader),
                None,
                PrimitiveTopology::TriangleList,
                CullModeFlags::NONE, FrontFace::Clockwise,
                DepthHandling::Some(true, false), // test, dont write
                vec![BlendMode::Off])?;

        let mut shade_gfx = ShadeGfx {
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
            descriptor_set: descriptor_set,
            desc_layout: desc_layout,
            material_image_view: material_image_view,
            normals_image_view: normals_image_view,
            diffuse_image_view: diffuse_image_view,
            sampler: sampler,
            directional_lights: HashMap::new(),
            next_dlight_token: 0,
        };

        shade_gfx.write();

        Ok(shade_gfx)

    }

    pub fn rebuild(&mut self, device: &Device, target_data: &TargetData)
        -> Result<()>
    {
        self.diffuse_image_view = target_data.diffuse_image.
            get_image_view(device)?;
        self.normals_image_view = target_data.normals_image.
            get_image_view(device)?;
        self.material_image_view = target_data.material_image.
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
                                image_view: Some(self.diffuse_image_view.clone()),
                                image_layout: ImageLayout::ShaderReadOnlyOptimal,
                            }
                        ]
                    ),
                    chain: None,
                },
                WriteDescriptorSet {
                    dst_set: self.descriptor_set.clone(),
                    dst_binding: 1,
                    dst_array_element: 0, // only have 1 element
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    elements: WriteDescriptorSetElements::ImageInfo(
                        vec![
                            DescriptorImageInfo {
                                sampler: Some(self.sampler.clone()),
                                image_view: Some(self.normals_image_view.clone()),
                                image_layout: ImageLayout::ShaderReadOnlyOptimal,
                            }
                        ]
                    ),
                    chain: None,
                },
                WriteDescriptorSet {
                    dst_set: self.descriptor_set.clone(),
                    dst_binding: 2,
                    dst_array_element: 0, // only have 1 element
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    elements: WriteDescriptorSetElements::ImageInfo(
                        vec![
                            DescriptorImageInfo {
                                sampler: Some(self.sampler.clone()),
                                image_view: Some(self.material_image_view.clone()),
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

    pub fn record(&self, command_buffer: CommandBuffer,
                  params_desc_set: DescriptorSet)
    {
        // Bind our pipeline
        command_buffer.bind_pipeline(PipelineBindPoint::Graphics, &self.pipeline);

        // FIXME: bind shading_image as a texture desc set
        command_buffer.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            &self.pipeline_layout,
            0, // starting with first set
            &[self.descriptor_set.clone(),
              params_desc_set],
            None,
        );

        command_buffer.draw(3, 1, 0, 0);
    }

    pub fn add_directional_light(&mut self, direction: Vec3<f32>, irradiance: Vec3<f32>)
                                 -> u32
    {
        let token: u32 = self.next_dlight_token;
        self.next_dlight_token += 1;

        self.directional_lights.insert(token, DLight {
            direction: direction,
            irradiance: irradiance
        });

        token
    }

    pub fn change_directional_light(&mut self, token: u32,
                                    direction: Vec3<f32>, irradiance: Vec3<f32>)
        -> Result<()>
    {
        if let Some(dl) = self.directional_lights.get_mut(&token) {
            dl.direction = direction;
            dl.irradiance = irradiance;
        } else {
            return Err(ErrorKind::General("No such light.".to_owned()).into());
        }
        Ok(())
    }

    pub fn remove_directional_light(&mut self, token: u32)
    {
        self.directional_lights.remove(&token);
    }
}

fn vertex_shader(device: &Device) -> Result<ShaderModule>
{
    let bytes: &[u8] = glsl_vs!(r#"
#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) out vec2 outUV;

out gl_PerVertex
{
  vec4 gl_Position;
};

void main()
{
  // We are rendering 1 full triangle which covers the entire screen
  // and goes beyond the screen. This trick was used by Sascha Willems
  // and also described by Bill Bilodeau from AMD as being faster
  // than a quad.

  // (0, 0),
  // (2, 0),
  // (0, 2)
  outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);

  // (-1.p, -1.0, 0.0, 1.0)
  // ( 1.0,  3.0, 0.0, 1.0)
  // ( 3.0,  1.0, 0.0, 1.0)
  gl_Position = vec4(outUV * 2.0f - 1.0f, 0.0f, 1.0f);
}
"#);

    let create_info = ShaderModuleCreateInfo {
        flags: ShaderModuleCreateFlags::empty(),
        code: bytes.to_vec(),
        chain: None,
    };

    Ok(device.create_shader_module(&create_info, None)?)
}

fn fragment_shader(device: &Device)
                   -> Result<ShaderModule>
{
    let bytes: &[u8] = glsl_fs!(r#"#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 0, binding = 0) uniform ParamsUBO {
  float bloom_strength;
  float bloom_scale;
  float blur_level;
  float white_point;
} params;

layout (set = 1, binding = 0) uniform DLightsUBO {
  vec4 directions[];
  vec4 irradiances[];
} dlights;

layout (binding = 0) uniform sampler2D diffusemap;  // A2B10G10R10_UNorm_Pack32
layout (binding = 1) uniform sampler2D normalsmap;  // A2B10G10R10_UNorm_Pack32
layout (binding = 2) uniform sampler2D materialmap; // R8G8B8_UNorm
//GINA FIXME WE NEED DEPTH BUFFER READS

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_color; // can be >1.0, post will handle it.

void main() {
  // FIXME
  out_color = texture(diffusemap, uv);
}
"#);

    let create_info = ShaderModuleCreateInfo {
        flags: ShaderModuleCreateFlags::empty(),
        code: bytes.to_vec(),
        chain: None,
    };

    Ok(device.create_shader_module(&create_info, None)?)
}
