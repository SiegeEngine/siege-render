
pub use siege_mesh::{ColoredVertex, StandardVertex, GuiRectangleVertex, GrayboxVertex,
                     CheapV1Vertex, CheapV2Vertex, StarVertex, CubemapVertex};
use dacite::core::{PipelineVertexInputStateCreateInfo, Format};

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
    ($base: path, $field: ident) => {
        {
            #[allow(unused_unsafe)]
            unsafe {
                let b: $base = ::std::mem::MaybeUninit::uninit().assume_init();
                (&b.$field as *const _ as isize) - (&b as *const _ as isize)
            }
        }
    }
}

pub trait VulkanVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo;
}

impl VulkanVertex for ColoredVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<ColoredVertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(ColoredVertex, pos) as u32,
                },
                VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(ColoredVertex, color) as u32,
                },
                VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(ColoredVertex, normal) as u32,
                },
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for StandardVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<StandardVertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(StandardVertex, pos) as u32,
                },
                VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(StandardVertex, normal) as u32,
                },
                VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: Format::R32G32_SFloat,
                    offset: offset_of!(StandardVertex, uv) as u32,
                },
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for GuiRectangleVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<GuiRectangleVertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32_SFloat,
                    offset: offset_of!(GuiRectangleVertex, pos) as u32,
                },
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for GrayboxVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<GrayboxVertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(GrayboxVertex, pos) as u32,
                },
                VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(GrayboxVertex, normal) as u32,
                },
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for CheapV1Vertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<CheapV1Vertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(CheapV1Vertex, pos) as u32,
                },
                VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: Format::R32G32_SFloat,
                    offset: offset_of!(CheapV1Vertex, uv) as u32,
                },
                VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: Format::R32_SFloat,
                    offset: offset_of!(CheapV1Vertex, shininess) as u32,
                },
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for CheapV2Vertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<CheapV2Vertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(CheapV2Vertex, pos) as u32,
                },
                VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: Format::R32G32_SFloat,
                    offset: offset_of!(CheapV2Vertex, uv) as u32,
                },
                VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(CheapV2Vertex, normal) as u32,
                },
                VertexInputAttributeDescription {
                    location: 3,
                    binding: 0,
                    format: Format::R32_SFloat,
                    offset: offset_of!(CheapV2Vertex, shininess) as u32,
                },
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for StarVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<StarVertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(StarVertex, pos) as u32,
                },
                VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(StarVertex, xyz) as u32,
                }
            ],
            chain: None,
        }
    }
}

impl VulkanVertex for CubemapVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
        use dacite::core::{VertexInputBindingDescription,
                           VertexInputRate, VertexInputAttributeDescription};

        PipelineVertexInputStateCreateInfo {
            flags: Default::default(),
            vertex_binding_descriptions: vec![
                VertexInputBindingDescription {
                    binding: 0_u32,
                    stride: ::std::mem::size_of::<CubemapVertex>() as u32,
                    input_rate: VertexInputRate::Vertex,
                },
            ],
            vertex_attribute_descriptions: vec![
                VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: Format::R32G32B32_SFloat,
                    offset: offset_of!(CubemapVertex, pos) as u32,
                },
            ],
            chain: None,
        }
    }
}
