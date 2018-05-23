use ash::vk::types::{Format, PipelineVertexInputStateCreateInfo, StructureType,
                     VertexInputAttributeDescription, VertexInputBindingDescription,
                     VertexInputRate};
pub use siege_mesh::{CheapV1Vertex, CheapV2Vertex, ColoredVertex, CubemapVertex, GrayboxVertex,
                     GuiRectangleVertex, StandardVertex, StarVertex};
use std::ptr;

/// A trait for declaring a vertex structure to Vulkan. Implementers of this trait
/// should be very aware that `PipelineVertexInputStateCreateInfo` contains raw
/// pointers, and these must point to valid memory which remains valid, and therefore
/// must be static (use a lazy_static like we do).
pub trait VulkanVertex {
    fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo;
}

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = ::std::mem::uninitialized();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

macro_rules! impl_vulkan_vertex {
    ($typ:ty, $static_bd:expr, $static_ad:expr) => {
        impl VulkanVertex for $typ {
            fn get_input_state_create_info() -> PipelineVertexInputStateCreateInfo {
                PipelineVertexInputStateCreateInfo {
                    s_type: StructureType::PipelineVertexInputStateCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    vertex_binding_description_count: $static_bd.len() as u32,
                    p_vertex_binding_descriptions: $static_bd.as_ptr(),
                    vertex_attribute_description_count: $static_ad.len() as u32,
                    p_vertex_attribute_descriptions: $static_ad.as_ptr(),
                }
            }
        }
    };
}

static COLORED_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<ColoredVertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref COLORED_VERTEX_AD: [VertexInputAttributeDescription; 3] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(ColoredVertex, pos) as u32,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(ColoredVertex, color) as u32,
        },
        VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(ColoredVertex, normal) as u32,
        },
    ];
}

impl_vulkan_vertex!(ColoredVertex, COLORED_VERTEX_BD, COLORED_VERTEX_AD);

static STANDARD_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<StandardVertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref STANDARD_VERTEX_AD: [VertexInputAttributeDescription; 3] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(StandardVertex, pos) as u32,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(StandardVertex, normal) as u32,
        },
        VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: Format::R32g32Sfloat,
            offset: offset_of!(StandardVertex, uv) as u32,
        },
    ];
}

impl_vulkan_vertex!(StandardVertex, STANDARD_VERTEX_BD, STANDARD_VERTEX_AD);

static GUI_RECTANGLE_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<GuiRectangleVertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref GUI_RECTANGLE_VERTEX_AD: [VertexInputAttributeDescription; 1] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32Sfloat,
            offset: offset_of!(GuiRectangleVertex, pos) as u32,
        },
    ];
}

impl_vulkan_vertex!(
    GuiRectangleVertex,
    GUI_RECTANGLE_VERTEX_BD,
    GUI_RECTANGLE_VERTEX_AD
);

static GRAYBOX_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<GrayboxVertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref GRAYBOX_VERTEX_AD: [VertexInputAttributeDescription; 2] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(GrayboxVertex, pos) as u32,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(GrayboxVertex, normal) as u32,
        },
    ];
}

impl_vulkan_vertex!(GrayboxVertex, GRAYBOX_VERTEX_BD, GRAYBOX_VERTEX_AD);

static CHEAP_V1_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<CheapV1Vertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref CHEAP_V1_VERTEX_AD: [VertexInputAttributeDescription; 3] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(CheapV1Vertex, pos) as u32,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32g32Sfloat,
            offset: offset_of!(CheapV1Vertex, uv) as u32,
        },
        VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: Format::R32Sfloat,
            offset: offset_of!(CheapV1Vertex, shininess) as u32,
        },
    ];
}

impl_vulkan_vertex!(CheapV1Vertex, CHEAP_V1_VERTEX_BD, CHEAP_V1_VERTEX_AD);

static CHEAP_V2_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<CheapV2Vertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref CHEAP_V2_VERTEX_AD: [VertexInputAttributeDescription; 4] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(CheapV2Vertex, pos) as u32,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32g32Sfloat,
            offset: offset_of!(CheapV2Vertex, uv) as u32,
        },
        VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(CheapV2Vertex, normal) as u32,
        },
        VertexInputAttributeDescription {
            location: 3,
            binding: 0,
            format: Format::R32Sfloat,
            offset: offset_of!(CheapV2Vertex, shininess) as u32,
        },
    ];
}

impl_vulkan_vertex!(CheapV2Vertex, CHEAP_V2_VERTEX_BD, CHEAP_V2_VERTEX_AD);

static STAR_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<StarVertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref STAR_VERTEX_AD: [VertexInputAttributeDescription; 2] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(StarVertex, pos) as u32,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(StarVertex, xyz) as u32,
        },
    ];
}

impl_vulkan_vertex!(StarVertex, STAR_VERTEX_BD, STAR_VERTEX_AD);

static CUBEMAP_VERTEX_BD: [VertexInputBindingDescription; 1] = [
    VertexInputBindingDescription {
        binding: 0_u32,
        stride: ::std::mem::size_of::<CubemapVertex>() as u32,
        input_rate: VertexInputRate::Vertex,
    },
];

lazy_static! {
    static ref CUBEMAP_VERTEX_AD: [VertexInputAttributeDescription; 1] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32g32b32Sfloat,
            offset: offset_of!(CubemapVertex, pos) as u32,
        },
    ];
}

impl_vulkan_vertex!(CubemapVertex, CUBEMAP_VERTEX_BD, CUBEMAP_VERTEX_AD);
