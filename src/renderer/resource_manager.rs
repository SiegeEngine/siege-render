
use errors::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use dacite::core::{Device, ShaderModule};

use siege_mesh::{VertexType,
                 GrayboxVertex, CubemapVertex};
//ColoredVertex, StandardVertex, GuiRectangleVertex
//CheapV1Vertex, CheapV2Vertex
use super::buffer::HostVisibleBuffer;
use super::image_wrap::{ImageWrap, ImageWrapType};
use super::memory::Memory;
use super::commander::Commander;
use super::mesh::VulkanMesh;

pub struct ResourceManager {
    asset_path: PathBuf,
    shaders: HashMap<String, ShaderModule>,

    // We can't use type parameterization for meshes, we have to enumerate them.
    // (a trait with type erasure doesn't give us enough power)
    // We split them up by vertex type, but possibly we want to split them up by
    // pipeline.
//    colored_meshes: HashMap<String, VulkanMesh<ColoredVertex>>,
//    standard_meshes: HashMap<String, VulkanMesh<StandardVertex>>,
//    gui_rectangle_meshes: HashMap<String, VulkanMesh<GuiRectangleVertex>>,
    graybox_meshes: HashMap<String, VulkanMesh<GrayboxVertex>>,
//    cheap_v1_meshes: HashMap<String, VulkanMesh<CheapV1Vertex>>,
//    cheap_v2_meshes: HashMap<String, VulkanMesh<CheapV2Vertex>>,
    cubemap_meshes: HashMap<String, VulkanMesh<CubemapVertex>>,

    textures: HashMap<String, ImageWrap>,
}

impl ResourceManager {
    pub fn new(asset_path: PathBuf) -> ResourceManager
    {
        ResourceManager {
            asset_path: asset_path,
            shaders: HashMap::new(),
//            colored_meshes: HashMap::new(),
//            standard_meshes: HashMap::new(),
//            gui_rectangle_meshes: HashMap::new(),
            graybox_meshes: HashMap::new(),
//            cheap_v1_meshes: HashMap::new(),
//            cheap_v2_meshes: HashMap::new(),
            cubemap_meshes: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn load_shader(&mut self, device: &Device, name: &str) -> Result<ShaderModule>
    {
        use dacite::core::{ShaderModuleCreateInfo, ShaderModuleCreateFlags};

        if let Some(s) = self.shaders.get(name) {
            return Ok(s.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("shaders/{}.spv", name));
        let shader_spv_file = File::open(&path)?;
        // FIXME: this just skips bad bytes, rather than erroring
        let bytes: Vec<u8> = shader_spv_file.bytes()
            .filter_map(|byte| byte.ok())
            .collect();

        let create_info = ShaderModuleCreateInfo {
            flags: ShaderModuleCreateFlags::empty(),
            code: bytes,
            chain: None,
        };

        let shader_module = device.create_shader_module(&create_info, None)?;

        self.shaders.insert(name.to_owned(), shader_module.clone());

        Ok(shader_module)
    }

    /*
    pub fn load_colored_mesh(&mut self,
                             device: &Device,
                             memory: &mut Memory,
                             commander: &Commander,
                             staging_buffer: &HostVisibleBuffer<u8>,
                             name: &str)
                             -> Result<VulkanMesh<ColoredVertex>>
    {
        // Check if we already have it
        if let Some(m) = self.colored_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/colored/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::Colored {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_colored(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.colored_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }
     */

    /*
    pub fn load_standard_mesh(&mut self,
                              device: &Device,
                              memory: &mut Memory,
                              commander: &Commander,
                              staging_buffer: &HostVisibleBuffer<u8>,
                              name: &str)
                              -> Result<VulkanMesh<StandardVertex>>
    {
        // Check if we already have it
        if let Some(m) = self.standard_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/standard/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::Standard {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_standard(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.standard_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }
     */

    /*
    pub fn load_gui_rectangle_mesh(&mut self,
                                   device: &Device,
                                   memory: &mut Memory,
                                   commander: &Commander,
                                   staging_buffer: &HostVisibleBuffer<u8>,
                                   name: &str)
                                   -> Result<VulkanMesh<GuiRectangleVertex>>
    {
        // Check if we already have it
        if let Some(m) = self.gui_rectangle_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/gui_rectangle/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::GuiRectangle {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_gui_rectangle(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.gui_rectangle_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }
     */

    pub fn load_graybox_mesh(&mut self,
                             device: &Device,
                             memory: &mut Memory,
                             commander: &Commander,
                             staging_buffer: &HostVisibleBuffer<u8>,
                             name: &str)
                             -> Result<VulkanMesh<GrayboxVertex>>
    {
        // Check if we already have it
        if let Some(m) = self.graybox_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/graybox/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::Graybox {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_graybox(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.graybox_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }

    /*
    pub fn load_cheap_v1_mesh(&mut self,
                              device: &Device,
                              memory: &mut Memory,
                              commander: &Commander,
                              staging_buffer: &HostVisibleBuffer<u8>,
                              name: &str)
                              -> Result<VulkanMesh<CheapV1Vertex>>
    {
        // Check if we already have it
        if let Some(m) = self.cheap_v1_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/cheapv1/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::CheapV1 {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_cheapv1(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.cheap_v1_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }
     */

    /*
    pub fn load_cheap_v2_mesh(&mut self,
                              device: &Device,
                              memory: &mut Memory,
                              commander: &Commander,
                              staging_buffer: &HostVisibleBuffer<u8>,
                              name: &str)
                              -> Result<VulkanMesh<CheapV2Vertex>>
    {
        // Check if we already have it
        if let Some(m) = self.cheap_v2_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/cheapv2/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::CheapV2 {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_cheapv2(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.cheap_v2_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }
     */

    pub fn load_cubemap_mesh(&mut self,
                             device: &Device,
                             memory: &mut Memory,
                             commander: &Commander,
                             staging_buffer: &HostVisibleBuffer<u8>,
                             name: &str)
                             -> Result<VulkanMesh<CubemapVertex>>
    {
        // Check if we already have it
        if let Some(m) = self.cubemap_meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push(format!("meshes/cubemap/{}.mesh", name));
        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        if vertex_type != VertexType::Cubemap {
            return Err(ErrorKind::WrongVertexType.into());
        }
        let mesh = ::siege_mesh::deserialize_cubemap(&*bytes)?;
        let vulkan_mesh = VulkanMesh::new(device, memory, commander,
                                          staging_buffer,
                                          mesh, name)?;
        self.cubemap_meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }

    pub fn load_texture(
        &mut self,
        device: &Device,
        memory: &mut Memory,
        commander: &Commander,
        staging_buffer: &HostVisibleBuffer<u8>,
        name: &str)
        -> Result<ImageWrap>
    {
        // Check if we already have it
        if let Some(texref) = self.textures.get(name) {
            return Ok(texref.clone());
        }

        let mut path = self.asset_path.clone();

        // All textures under the siege engine are stored in DDS files
        // compressed with Zstd, and named with the ".dds.zst" extension.
        path.push(format!("textures/{}.dds.zst", name));
        let f = File::open(path)?;

        // Decompress
        use zstd::stream::Decoder;
        let mut d = Decoder::new(f)?;

        // Interpret as a DDS file
        use ddsfile::Dds;
        let dds = Dds::read(&mut d)?;

        // Determine format
        let (format, component_mapping) = {
            match dds.get_dxgi_format() {
                Some(dxgi_format) => {
                    match ::format::from_dxgi(dxgi_format) {
                        Some(f) => (f, ComponentMapping::identity()),
                        None => return Err(ErrorKind::UnsupportedFormat.into()),
                    }
                },
                None => match dds.get_d3d_format() {
                    Some(d3d_format) => {
                        match ::format::from_d3d(d3d_format) {
                            Some(pair) => pair,
                            None => return Err(ErrorKind::UnsupportedFormat.into()),
                        }
                    },
                    None => return Err(ErrorKind::UnsupportedFormat.into()),
                }
            }
        };

        use ddsfile::Caps2;
        let image_wrap_type = if dds.header.caps2.contains(Caps2::CUBEMAP) {
            ImageWrapType::Cubemap
        } else {
            ImageWrapType::Standard
        };
        let num_layers = dds.get_num_array_layers();

        use dacite::core::Extent3D;
        let extent = Extent3D {
            width: dds.get_width(),
            height: dds.get_height(),
            depth: dds.get_depth(),
        };

        // Copy texture to staging buffer
        let mut offset: u64 = 0;
        for layer in 0..num_layers {
            let data = dds.get_data(layer)?;
            staging_buffer.block.write(data, offset)?;
            offset += data.len() as u64;
        }

        // create image wrap
        use dacite::core::{ImageLayout, ImageTiling, ImageUsageFlags,
                           ComponentMapping};
        use super::memory::Lifetime;
        let mut image_wrap = ImageWrap::new(
            device, memory, format, component_mapping,
            extent,
            image_wrap_type,
            ImageLayout::Undefined,
            ImageTiling::Optimal,
            ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED,
            Lifetime::Temporary,
            &*format!("texture {}", name))?;

        // copy_in_from_buffer
        // (this will transition to ImageLayout::TransferDstOptimal first)
        image_wrap.copy_in_from_buffer(
            device,
            commander.xfr_command_buffer.clone(),
            commander.xfr_queue.clone(),
            &staging_buffer.buffer)?;

        // transfer layout to ImageLayout::ShaderReadOnlyOptimal
        use dacite::core::{AccessFlags, ImageAspectFlags, OptionalMipLevels,
                           OptionalArrayLayers, ImageSubresourceRange,
                           PipelineStageFlags};
        image_wrap.transition_layout_now(
            device, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::TRANSFER_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::TRANSFER, PipelineStageFlags::VERTEX_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(num_layers),
            },
            commander.early_command_buffers[0].clone(),
            commander.early_queue.clone())?;
        // And wait until that completes
        // FIXME - this is a CPU side stall.

        // insert to hashmap
        self.textures.insert(name.to_owned(), image_wrap.clone());

        Ok(image_wrap)
    }
}
