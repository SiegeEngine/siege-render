
use dacite::core::{Device, DescriptorPool, DescriptorSet, DescriptorSetLayout,
                   DescriptorSetLayoutBinding, ImageView, ImageLayout, Sampler,
                   DescriptorType, CommandBuffer, RenderPass, Viewport, Rect2D,
                   PipelineBindPoint, Pipeline, PipelineLayout, PrimitiveTopology,
                   CullModeFlags, FrontFace, ShaderModuleCreateFlags,
                   ShaderModuleCreateInfo, ShaderModule};
use error::*;
use super::target_data::TargetData;
use super::{DepthHandling, BlendMode};

pub struct BlurGfx {
    pipeline_v: Pipeline,
    pipeline_layout_v: PipelineLayout,
    pipeline_h: Pipeline,
    pipeline_layout_h: PipelineLayout,
    descriptor_set_v: DescriptorSet,
    descriptor_set_h: DescriptorSet,
    #[allow(dead_code)]
    desc_layout: DescriptorSetLayout,
    shading_image_view: ImageView,
    blur_image_view: ImageView,
    sampler: Sampler,
}

impl BlurGfx {
    pub fn new(device: &Device,
               descriptor_pool: DescriptorPool,
               target_data: &TargetData,
               blurh_render_pass: RenderPass,
               blurv_render_pass: RenderPass,
               viewport: Viewport,
               scissors: Rect2D,
               params_layout: DescriptorSetLayout)
               -> Result<BlurGfx, Error>
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

        let blur_image_view = target_data.blur_image.
            get_image_view(device)?;

        let desc_bindings = {
            use dacite::core::ShaderStageFlags;
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

        let (descriptor_set_h, descriptor_set_v) = {
            use dacite::core::DescriptorSetAllocateInfo;

            let alloc_info = DescriptorSetAllocateInfo {
                descriptor_pool: descriptor_pool.clone(),
                set_layouts: vec![
                    desc_layout.clone(),
                    desc_layout.clone()
                ],
                chain: None,
            };

            let mut descriptor_sets = DescriptorPool::allocate_descriptor_sets(&alloc_info)?;

            let dsh = descriptor_sets.pop().unwrap();
            let dsv = descriptor_sets.pop().unwrap();
            (dsh, dsv)
        };

        let vertex_shader_h = vertex_shader_h(device)?;
        let fragment_shader_h = fragment_shader_h(device)?;

        let (pipeline_layout_h, pipeline_h) =
            super::pipeline::create(
                device, viewport, scissors,
                true, // reversed depth buffer irrelevant for blur
                blurh_render_pass, vec![
                    desc_layout.clone(),
                    params_layout.clone(),
                ],
                Some(vertex_shader_h), None, Some(fragment_shader_h), None,
                None,
                PrimitiveTopology::TriangleList,
                CullModeFlags::NONE, FrontFace::Clockwise,
                DepthHandling::None,
                vec![BlendMode::Off],
                vec![])?;

        let vertex_shader_v = vertex_shader_v(device)?;
        let fragment_shader_v = fragment_shader_v(device)?;

        let (pipeline_layout_v, pipeline_v) =
            super::pipeline::create(
                device, viewport, scissors,
                true, // reversed depth buffer irrelevant for blur
                blurv_render_pass, vec![
                    desc_layout.clone(),
                    params_layout.clone()],
                Some(vertex_shader_v), None, Some(fragment_shader_v), None,
                None,
                PrimitiveTopology::TriangleList,
                CullModeFlags::NONE, FrontFace::Clockwise,
                DepthHandling::None,
                vec![BlendMode::Add],
                vec![])?;

        let mut blur_gfx = BlurGfx {
            pipeline_v: pipeline_v,
            pipeline_layout_v: pipeline_layout_v,
            pipeline_h: pipeline_h,
            pipeline_layout_h: pipeline_layout_h,
            descriptor_set_v: descriptor_set_v,
            descriptor_set_h: descriptor_set_h,
            desc_layout: desc_layout,
            shading_image_view: shading_image_view,
            blur_image_view: blur_image_view,
            sampler: sampler
        };

        blur_gfx.write();

        Ok(blur_gfx)
    }

    pub fn rebuild(&mut self, device: &Device, target_data: &TargetData)
        -> Result<(), Error>
    {
        self.shading_image_view = target_data.shading_image.
            get_image_view(device)?;
        self.blur_image_view = target_data.blur_image.
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
                    dst_set: self.descriptor_set_h.clone(),
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
                WriteDescriptorSet {
                    dst_set: self.descriptor_set_v.clone(),
                    dst_binding: 0,
                    dst_array_element: 0, // only have 1 element
                    descriptor_type: DescriptorType::CombinedImageSampler,
                    elements: WriteDescriptorSetElements::ImageInfo(
                        vec![
                            DescriptorImageInfo {
                                sampler: Some(self.sampler.clone()),
                                image_view: Some(self.blur_image_view.clone()),
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

    pub fn record_blurh(&self, command_buffer: CommandBuffer,
                        params_desc_set: DescriptorSet)
    {
        // Bind our pipeline
        command_buffer.bind_pipeline(PipelineBindPoint::Graphics, &self.pipeline_h);

        command_buffer.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            &self.pipeline_layout_h,
            0, // starting with first set
            &[self.descriptor_set_h.clone(),
              params_desc_set],
            None,
        );

        command_buffer.draw(3, 1, 0, 0);
    }

    pub fn record_blurv(&self, command_buffer: CommandBuffer,
                        params_desc_set: DescriptorSet)
    {
        // Bind our pipeline
        command_buffer.bind_pipeline(PipelineBindPoint::Graphics, &self.pipeline_v);

        command_buffer.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            &self.pipeline_layout_v,
            0, // starting with first set
            &[self.descriptor_set_v.clone(),
              params_desc_set],
            None,
        );

        command_buffer.draw(3, 1, 0, 0);
    }
}

fn vertex_shader_h(device: &Device) -> Result<ShaderModule, Error>
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
	outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
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

fn fragment_shader_h(device: &Device) -> Result<ShaderModule, Error>
{
    let bytes: &[u8] = glsl_fs!(r#"
#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (binding = 0) uniform sampler2D samplerColor;

layout (set = 1, binding = 0) uniform UBO
{
  mat4 inv_projection;
  vec4 dlight_directions[2];
  vec4 dlight_irradiances[2];
  float bloom_strength;
  float bloom_cliff;
  float blur_level;
  float white_point;
} ubo;

layout (location = 0) in vec2 inUV;

layout (location = 0) out vec4 outFragColor;

// NOTE: Because we are filter-sampling and blurring at once, we are filter-sampling
// about 10x as much as we really need to (if we had another render pass and another
// target to hold the bright pixels only).

// Bright pass filter sampling
vec3 samp(vec2 offset) {

  // Get the texture irradiance value
  vec3 color = texture(samplerColor, inUV + offset).rgb;

  // Convert to xyz
  /*
  mat3 rgb2xyz = mat3( // column major order
    0.4124, 0.2126, 0.0193,
    0.3576, 0.7152, 0.1192,
    0.1805, 0.0722, 0.9505);
  vec3 xyz = rgb2xyz * color;
  */

  // New function, considers some lums will be very high
  // Output is in range [0,1]
  //  float mult = 1 - pow(1.08, -color.y);
  float mult = color.y / (color.y + 6);

  // Adjust based on the bloom strength
  // Output will be in range [0,bloom_strength]
  mult *= ubo.bloom_strength;

  // Also include blur factor
  mult = clamp(mult + ubo.blur_level, 0, 1);

  // Scale the luminance
  color *= mult;

  // We have to cap the maximum values
  // (If we don't we will overbloom out to rectangles)
  color = color / (color + 1); // reinhard
  // color = clamp(color, 0.0, 1.0); // clamp

  // Convert back to RGB
/*  mat3 xyz2rgb = mat3( // column major order
    3.2406255, -0.96893071, 0.055710120,
    -1.5372080, 1.8757561, -0.20402105,
    -0.49862860, 0.041517524, 1.0569959);
  vec3 rgb = xyz2rgb * xyz;
*/

  return color;
}

void main()
{
  float weight[6];
  // We have a sharp falloff at the first pixel, because this looks nice for stars
  // and doesn't make other things look awful.
  weight[0] = 1.0;
  weight[1] = 0.8225776;
  weight[2] = 0.45783338;
  weight[3] = 0.17242163;
  weight[4] = 0.04393694;
  weight[5] = 0.0075756805;

  vec2 tex_offset = 1.0 / textureSize(samplerColor, 0); // gets size of single texel
  vec3 result = samp(vec2(0.0, 0.0)) * weight[0]; // current fragment's contribution
  for (int i = 1; i < 6; ++i) {
    result += samp(vec2(tex_offset.x * i, 0.0)) * weight[i] * ubo.bloom_cliff;
    result += samp(vec2(-tex_offset.x * i, 0.0)) * weight[i] * ubo.bloom_cliff;
  }
  // do not go beyond maximum f16
  outFragColor = vec4(min(result, 65504), 1.0);
}
"#);

    let create_info = ShaderModuleCreateInfo {
        flags: ShaderModuleCreateFlags::empty(),
        code: bytes.to_vec(),
        chain: None,
    };

    Ok(device.create_shader_module(&create_info, None)?)
}

fn vertex_shader_v(device: &Device) -> Result<ShaderModule, Error>
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
	outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
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

fn fragment_shader_v(device: &Device) -> Result<ShaderModule, Error>
{
    let bytes: &[u8] = glsl_fs!(r#"
#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 0, binding = 0) uniform sampler2D samplerColor;

layout (set = 1, binding = 0) uniform UBO
{
  mat4 inv_projection;
  vec4 dlight_directions[2];
  vec4 dlight_irradiances[2];
  float bloom_strength;
  float bloom_cliff;
  float blur_level;
  float white_point;
} ubo;

layout (location = 0) in vec2 inUV;

layout (location = 0) out vec4 outFragColor;

vec3 samp(vec2 offset) {
  return texture(samplerColor, inUV + offset).rgb;
}

void main()
{
  float weight[6];
  weight[0] = 1.0;
  weight[1] = 0.8225776;
  weight[2] = 0.45783338;
  weight[3] = 0.17242163;
  weight[4] = 0.04393694;
  weight[5] = 0.0075756805;

  vec2 tex_offset = 1.0 / textureSize(samplerColor, 0); // gets size of single texel
  vec3 result = samp(vec2(0.0, 0.0)) * weight[0]; // current fragment's contribution
  for (int i = 1; i < 6; ++i) {
    result += samp(vec2(0.0, tex_offset.y * i)) * weight[i] * ubo.bloom_cliff;
    result += samp(vec2(0.0, -tex_offset.y * i)) * weight[i] * ubo.bloom_cliff;
  }
  outFragColor = vec4(result, 1.0);
}
"#);

    let create_info = ShaderModuleCreateInfo {
        flags: ShaderModuleCreateFlags::empty(),
        code: bytes.to_vec(),
        chain: None,
    };

    Ok(device.create_shader_module(&create_info, None)?)
}
