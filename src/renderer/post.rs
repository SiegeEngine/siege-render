
use dacite::core::{Device, DescriptorPool, DescriptorSet, DescriptorSetLayout,
                   DescriptorSetLayoutBinding, Sampler, ImageView, ImageLayout,
                   DescriptorType, CommandBuffer, RenderPass, Viewport, Rect2D,
                   PipelineBindPoint, Pipeline, PipelineLayout, PrimitiveTopology,
                   CullModeFlags, FrontFace, ShaderModuleCreateFlags,
                   ShaderModuleCreateInfo, ShaderModule};
use errors::*;
use super::target_data::TargetData;
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
               render_pass: RenderPass,
               viewport: Viewport,
               scissors: Rect2D,
               display_luminance: u32)
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

        let fragment_shader = fragment_shader(device, display_luminance)?;

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
                BlendMode::Off)?;

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

fn fragment_shader(device: &Device, display_luminance: u32) -> Result<ShaderModule>
{

    // Incorporate display luminance (output function exposure)
    let output_exposure: f32 = 100.0 / (display_luminance as f32);

    let code = format!("{}{}{}", FS_PREFIX, output_exposure, FS_SUFFIX);
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

layout (binding = 0) uniform sampler2D shadingTex;

layout (location = 0) in vec2 inUV;

layout (location = 0) out vec4 outFragColor;

float hlg(float scene_referred) {
  //r = reference white level (0.5)
  //a = 0.17883277,
  //b = 0.28466892,
  //c = 0.55991073

  if (scene_referred <= 1) {
    return min(1.0, 0.5 * sqrt(scene_referred));
  } else {
    return min(1.0, 0.17883277 * log(scene_referred - 0.28466892) + 0.55991073);
  }
}

vec3 ACESFilmApprox(vec3 x) {
  const float A = 2.51;
  const float B = 0.03;
  const float C = 2.43;
  const float D = 0.59;
  const float E = 0.14;
  return clamp( (x*(A*x+B)) / (x*(C*x+D)+E), 0.0, 1.0);
}

// FIXME - we need to take display_luminance into account for HDR displays.

void main()
{
  // Load scene referred color from shadingTex
  vec3 scene_referred = texture(shadingTex, inUV).rgb;

  // Tone Mapping
  // 1) Reinhard:
  // vec3 mapped = scene_referred / (scene_referred + vec3(1.0));
  //
  // 2) Exposure tone mapping:
  // const float exposure = 1.0;
  // vec3 mapped = vec3(1.0) - exp(-scene_referred * exposure);
  //
  // 3) Hybrid Log-Gamma (HLG):
  // vec3 mapped = vec3(hlg(scene_referred.x), hlg(scene_referred.y), hlg(scene_referred.z));
  //
  // 4) ACES filmic (approx):
  const float output_exposure = "#;


const FS_SUFFIX: &'static str = r#";
  vec3 mapped = ACESFilmApprox(output_exposure * scene_referred);

  outFragColor = vec4(mapped, 1.0);
}
"#;
