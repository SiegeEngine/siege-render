
use dacite::core::{Device, DescriptorPool, DescriptorSet, DescriptorSetLayout,
                   DescriptorSetLayoutBinding, Sampler, ImageView, ImageLayout,
                   DescriptorType, CommandBuffer, RenderPass, Viewport, Rect2D,
                   PipelineBindPoint, Pipeline, PipelineLayout, PrimitiveTopology,
                   CullModeFlags, FrontFace, ShaderModuleCreateFlags,
                   ShaderModuleCreateInfo, ShaderModule,
                   SpecializationInfo, SpecializationMapEntry};
use errors::*;
use super::target_data::TargetData;
use super::{DepthHandling, BlendMode};

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Tonemapper {
    Clamp,
    Reinhard,
    Exposure,
    HybridLogGamma,
    Falsecolor,
}

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
               render_pass: RenderPass,
               viewport: Viewport,
               scissors: Rect2D,
               display_luminance: u32,
               tonemapper: Tonemapper,
               params_layout: DescriptorSetLayout,
               surface_needs_gamma: bool)
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

        let vertex_shader = vertex_shader(device)?;

        let fragment_shader = fragment_shader(device, display_luminance, tonemapper)?;

        let fragment_spec = SpecializationInfo {
            map_entries: vec![
                SpecializationMapEntry { // near depth
                    constant_id: 0,
                    offset: 0,
                    size: 4,
                },
            ],
            data: if surface_needs_gamma {
                vec![ 0x01, 0x00, 0x00, 0x00 ]
            } else {
                vec![ 0x00, 0x00, 0x00, 0x00 ]
            }
        };

        let (pipeline_layout, pipeline) =
            super::pipeline::create(
                device, viewport, scissors,
                true, // reversed depth buffer irrelevant for post
                render_pass, vec![desc_layout.clone(),
                                  params_layout],
                Some(vertex_shader), None, Some(fragment_shader), Some(fragment_spec),
                None,
                PrimitiveTopology::TriangleList,
                CullModeFlags::NONE, FrontFace::Clockwise,
                DepthHandling::None,
                vec![BlendMode::Off])?;

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

fn fragment_shader(device: &Device, _display_luminance: u32,
                   tonemapper: Tonemapper)
                   -> Result<ShaderModule>
{
    // FIXME: incorporate display luminance
    //    GINA FIXME -- SET TRANSFER FUNCTION TO ACCOUNT FOR config.display_luminance
    //    let white_point = 80.0 / (display_luminance as f32);

    let code = format!("{}{}{}",
                       FS_PREFIX,
                       match tonemapper {
                           Tonemapper::Clamp => FS_TONEMAP_CLAMP,
                           Tonemapper::Reinhard => FS_TONEMAP_REINHARD,
                           Tonemapper::Exposure => FS_TONEMAP_EXPOSURE,
                           Tonemapper::HybridLogGamma => FS_TONEMAP_HLG,
                           Tonemapper::Falsecolor => FS_TONEMAP_FALSECOLOR,
                       },
                       FS_SUFFIX);

    use std::fs::File;
    use std::io::Read;

    let mut output: File =
        ::glsl_to_spirv::compile(&*code, ::glsl_to_spirv::ShaderType::Fragment)?;
    let mut bytes: Vec<u8> = Vec::new();
    output.read_to_end(&mut bytes)?;

    let create_info = ShaderModuleCreateInfo {
        flags: ShaderModuleCreateFlags::empty(),
        code: bytes,
        chain: None,
    };

    Ok(device.create_shader_module(&create_info, None)?)
}

const FS_PREFIX: &'static str = r#"#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(constant_id = 0) const int surface_needs_gamma = 0;

layout (binding = 0) uniform sampler2D shadingTex;

layout (set = 1, binding = 0) uniform UBO
{
  mat4 inv_projection;
  vec4 dlight_directions[2];
  vec4 dlight_irradiances[2];
  float bloom_strength;
  float bloom_scale;
  float blur_level;
  float white_point;
} ubo;

layout (location = 0) in vec2 inUV;

layout (location = 0) out vec4 outFragColor;
"#;

const FS_TONEMAP_HLG: &'static str = r#"
float hlg(float scene_referred) {
  const float r = 0.5; // reference white level
  const float a = 0.17883277;
  const float b = 0.28466892;
  const float c = 0.55991073;

  if (scene_referred <= 1) {
    return min(1.0, r * sqrt(scene_referred));
  } else {
    return min(1.0, a * log(scene_referred - b) + c);
  }
}

vec3 tonemap(vec3 scene_referred) {
  return vec3(hlg(scene_referred.r), hlg(scene_referred.g), hlg(scene_referred.b));
}
"#;

const FS_TONEMAP_FALSECOLOR: &'static str = r#"// False Color
vec3 tonemap(vec3 scene_referred) {
  vec3 colors[6] = vec3[](
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 1.0, 1.0),
    vec3(0.0, 1.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 1.0)
  );

  float lum = dot(vec3(0.2126729, 0.7151522, 0.0721750), scene_referred);
  float level = log2(lum/0.18);
  return colors[int(level) % 6];
}
"#;

const FS_TONEMAP_REINHARD: &'static str = r#"
vec3 tonemap(vec3 scene_referred) {
  return scene_referred / (scene_referred + vec3(1.0));
}
"#;

const FS_TONEMAP_CLAMP: &'static str = r#"
vec3 tonemap(vec3 scene_referred) {
  return clamp(scene_referred, 0.0, 1.0);
}
"#;

const FS_TONEMAP_EXPOSURE: &'static str = r#"
vec3 tonemap(vec3 scene_referred) {
  const float exposure = 1.0;
  return vec3(1.0) - exp(-scene_referred * exposure);
}
"#;

const FS_SUFFIX: &'static str = r#"

float srgb_gamma(float linear) {
  if (linear <= 0.0031308) {
    return 12.92 * linear;
  } else {
    return (1 + 0.055) * pow(linear, 1/2.4) - 0.055;
  }
}

void main()
{
  // Load scene referred color from shadingTex
  vec3 scene_referred = texture(shadingTex, inUV).rgb;

  vec3 tonemapped = tonemap(scene_referred);

  if (surface_needs_gamma != 0) {
    outFragColor = vec4(srgb_gamma(tonemapped.r),
                        srgb_gamma(tonemapped.g),
                        srgb_gamma(tonemapped.b),
                        1.0);
  } else {
    outFragColor = vec4(tonemapped, 1.0);
  }
}
"#;

