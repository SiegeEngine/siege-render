
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

pub struct ShadeGfx {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    #[allow(dead_code)] // this must remain alive
    desc_layout: DescriptorSetLayout,
    material_image_view: ImageView,
    normals_image_view: ImageView,
    diffuse_image_view: ImageView,
    depth_image_view: ImageView,
    sampler: Sampler,
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

        let depth_image_view = target_data.depth_image.get_image_view(device)?;
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
                DescriptorSetLayoutBinding {
                    binding: 3,
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

        let fragment_shader = fragment_shader(device)?;

        let fragment_spec = SpecializationInfo {
            map_entries: vec![
                SpecializationMapEntry { // near depth
                    constant_id: 0,
                    offset: 0,
                    size: 4,
                },
                SpecializationMapEntry { // far depth
                    constant_id: 0,
                    offset: 4,
                    size: 4,
                },
            ],
            // near than far
            data: if reversed_depth_buffer {
                vec![ 0x00, 0x00, 0x80, 0x3f, // 1.0 (0x3f800000) in LSB
                      0x00, 0x00, 0x00, 0x00 ]
            } else {
                vec![ 0x00, 0x00, 0x00, 0x00,
                      0x00, 0x00, 0x80, 0x3f ] // 1.0 (0x3f800000) in LSB
            }
        };

        let (pipeline_layout, pipeline) =
            super::pipeline::create(
                device, viewport, scissors,
                reversed_depth_buffer,
                render_pass, vec![desc_layout.clone(),
                                  params_layout],
                Some(vertex_shader), None, Some(fragment_shader), Some(fragment_spec),
                None,
                PrimitiveTopology::TriangleList,
                CullModeFlags::NONE, FrontFace::Clockwise,
                DepthHandling::None, // no depth attachment (we use as input herein)
                vec![BlendMode::Off],
                vec![])?;

        let mut shade_gfx = ShadeGfx {
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
            descriptor_set: descriptor_set,
            desc_layout: desc_layout,
            material_image_view: material_image_view,
            normals_image_view: normals_image_view,
            diffuse_image_view: diffuse_image_view,
            depth_image_view: depth_image_view,
            sampler: sampler,
        };

        shade_gfx.write();

        Ok(shade_gfx)

    }

    pub fn rebuild(&mut self, device: &Device, target_data: &TargetData)
        -> Result<()>
    {
        self.depth_image_view = target_data.depth_image.
            get_image_view(device)?;
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
                                image_view: Some(self.depth_image_view.clone()),
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
                                image_view: Some(self.diffuse_image_view.clone()),
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
                                image_view: Some(self.normals_image_view.clone()),
                                image_layout: ImageLayout::ShaderReadOnlyOptimal,
                            }
                        ]
                    ),
                    chain: None,
                },
                WriteDescriptorSet {
                    dst_set: self.descriptor_set.clone(),
                    dst_binding: 3,
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

  // (-1.0, -1.0, 0.0, 1.0)
  // ( 3.0, -1.0, 0.0, 1.0)
  // (-1.0,  3.0, 0.0, 1.0)
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
    let bytes: &[u8] = glsl_fs!(r#"
#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(constant_id = 0) const float depth_near = 0.0;
layout(constant_id = 1) const float depth_far = 1.0;

layout (set = 1, binding = 0) uniform ParamsUBO {
  mat4 inv_projection;
  vec4 dlight_directions[2];
  vec4 dlight_irradiances[2];
  float bloom_strength;
  float bloom_cliff;
  float blur_level;
  float ambient;
  float white_level;
} params;

layout (set = 0, binding = 0) uniform sampler2D depthbuffer; // D32_SFloat
layout (set = 0, binding = 1) uniform sampler2D diffusemap;  // A2B10G10R10_UNorm_Pack32
layout (set = 0, binding = 2) uniform sampler2D normalsmap;  // A2B10G10R10_UNorm_Pack32
layout (set = 0, binding = 3) uniform sampler2D materialmap; // R8G8B8_UNorm

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_color; // can be >1.0, post will handle it.

const float pi = 3.14159265359;

vec4 level(vec4 irrad) {
  return vec4(irrad.xyz / params.white_level, irrad.a);
}

vec3 shadingSpecularGGX(vec3 N, vec3 V, vec3 L, float roughness, vec3 F0)
{
    vec3 H = V + L;
    if (length(H) < 0.00001) { return vec3(0.0, 0.0, 0.0); }
    H = normalize(H);
    float dotLH = max(dot(L, H), 0.0);
    if (dotLH > 0.9999) { dotLH = 0.9999; } // head shadow spark fix
    float dotNH = max(dot(N, H), 0.0);
    float dotNL = max(dot(N, L), 0.0);
    float dotNV = max(dot(N, V), 0.0);
    float alpha = roughness * roughness;

    // D (GGX normal distribution, Trowbridge-Reitz)
    float alphaSqr = alpha * alpha;
    float denom = dotNH * dotNH * (alphaSqr - 1.0) + 1.0;
    if (denom==0.0) { return vec3(0.0, 0.0, 0.0); }
    float D = alphaSqr / (pi * denom * denom);
    // F (Fresnel term)
    float F_a = 1.0;
    float F_b = pow(1.0 - dotLH, 5); // manually?
    vec3 F = mix(vec3(F_b), vec3(F_a), F0);
    // G (remapped hotness, see Unreal Shading)
    float k = (alpha + 2 * roughness + 1) / 8.0;
    float G = dotNL / (mix(dotNL, 1, k) * mix(dotNV, 1, k));

    return D * F * G / 4.0;
}

vec4 decode_normal(vec4 n) {
  return normalize(vec4((n.xyz - 0.5) * 2, 0.0));
}

void main() {
  // Reconstruct view-space position of the fragment
  float fragdepth = texture(depthbuffer, uv).r;
  vec4 clipPos;
  clipPos.xy = (2.0 * uv) - 1;
  clipPos.z = (fragdepth - depth_near) / (depth_far - depth_near);
  clipPos.w = 1.0;
  vec4 position = params.inv_projection * clipPos;
  vec3 V = normalize(-position.xyz);

  // Sample the textures
  vec4 materials_sample = texture(materialmap, uv);
  float roughness = materials_sample.r;
  float metallicity = materials_sample.g;
  float ao = materials_sample.b;
  float cavity = materials_sample.a;
  vec3 albedo = texture(diffusemap, uv).rgb;
  vec3 N = decode_normal(texture(normalsmap, uv)).rgb;

  vec3 ambient_level = vec3(params.ambient, params.ambient, params.ambient);

  // Prepare terms we re-use
  vec3 diffuse = albedo * (1 - metallicity) * ao;

  // Start with ambient component
  vec3 color = ambient_level * diffuse;

  // Add each lights contribution
  for (int i=0; i<=1; i++) {
    if (params.dlight_irradiances[i].xyz == vec3(0.0, 0.0, 0.0)) {
      continue; // Do not process lights that are off.
    }
    vec3 light = params.dlight_irradiances[i].xyz;
    vec3 light_color = normalize(params.dlight_irradiances[i].xyz);
    float light_intensity = (params.dlight_irradiances[i].x +
      params.dlight_irradiances[i].y + params.dlight_irradiances[i].z) / 3.0;
    vec3 L = params.dlight_directions[i].xyz;
    float lambert = max(0.0, dot(L, N));

    // Add diffuse part
    color += diffuse * lambert * light;

    // Add specular part
    vec3 specular = mix(vec3(0.04), albedo, metallicity) * cavity * light_color;
    color += shadingSpecularGGX(N, V, L, roughness, specular) * light_intensity;
  }

  // Level the output (still allows >1.0 but sets base exposure/white_level)
  // Limit to maximum f16 to not wrap around.
  out_color = level(vec4(min(color, 65504), 1.0));
}
"#);

    let create_info = ShaderModuleCreateInfo {
        flags: ShaderModuleCreateFlags::empty(),
        code: bytes.to_vec(),
        chain: None,
    };

    Ok(device.create_shader_module(&create_info, None)?)
}
