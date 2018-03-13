
use errors::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use dacite::core::{Device, ShaderModule, BufferUsageFlags};

use siege_mesh::VertexType;
use super::buffer::{HostVisibleBuffer, DeviceLocalBuffer};
use super::image_wrap::{ImageWrap, ImageWrapType};
use super::memory::{Memory, Lifetime};
use super::commander::Commander;
use super::mesh::VulkanMesh;

pub struct ResourceManager {
    asset_path: PathBuf,
    shaders: HashMap<String, ShaderModule>,
    meshes: HashMap<String, VulkanMesh>,
    textures: HashMap<String, ImageWrap>,
    buffers: HashMap<String, DeviceLocalBuffer>,
}

impl ResourceManager {
    pub fn new(asset_path: PathBuf) -> ResourceManager
    {
        ResourceManager {
            asset_path: asset_path,
            shaders: HashMap::new(),
            meshes: HashMap::new(),
            textures: HashMap::new(),
            buffers: HashMap::new(),
        }
    }

    pub fn load_shader(&mut self, device: &Device, name: &str) -> Result<ShaderModule>
    {
        use dacite::core::{ShaderModuleCreateInfo, ShaderModuleCreateFlags};

        if let Some(s) = self.shaders.get(name) {
            return Ok(s.clone());
        }

        let mut path = self.asset_path.clone();
        path.push("shaders");
        path.push(format!("{}.spv", name));

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

    pub fn load_mesh(&mut self,
                     device: &Device,
                     memory: &mut Memory,
                     commander: &Commander,
                     staging_buffer: &mut HostVisibleBuffer,
                     dir: &str, // by type, e.g. 'graybox'
                     name: &str)
                     -> Result<VulkanMesh>
    {
        // Check if we already have it
        if let Some(m) = self.meshes.get(name) {
            return Ok(m.clone());
        }

        let mut path = self.asset_path.clone();
        path.push("meshes");
        path.push(dir);
        path.push(format!("{}.mesh", name));

        let (vertex_type, bytes) = ::siege_mesh::load_header(&path)?;
        let vulkan_mesh = {
            // FIXME: this per-vertex-type code is probably not required
            // anymore; will need to bubble up changes into siege-mesh.
            match vertex_type {
                VertexType::Colored => {
                    let mesh = ::siege_mesh::deserialize_colored(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::Standard => {
                    let mesh = ::siege_mesh::deserialize_standard(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::GuiRectangle => {
                    let mesh = ::siege_mesh::deserialize_gui_rectangle(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::Graybox => {
                    let mesh = ::siege_mesh::deserialize_graybox(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::CheapV1 => {
                    let mesh = ::siege_mesh::deserialize_cheapv1(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::CheapV2 => {
                    let mesh = ::siege_mesh::deserialize_cheapv2(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::Star => {
                    let mesh = ::siege_mesh::deserialize_star(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
                VertexType::Cubemap => {
                    let mesh = ::siege_mesh::deserialize_cubemap(&*bytes)?;
                    VulkanMesh::new(device, memory, commander,
                                    staging_buffer, mesh, name)?
                },
            }
        };

        self.meshes.insert(name.to_owned(), vulkan_mesh.clone());

        Ok(vulkan_mesh)
    }

    pub fn load_texture(
        &mut self,
        device: &Device,
        memory: &mut Memory,
        commander: &Commander,
        staging_buffer: &mut HostVisibleBuffer,
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
        path.push("textures");
        path.push(format!("{}.dds.zst", name));
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
        let mut offset: usize = 0;
        for layer in 0..num_layers {
            let data = dds.get_data(layer)?;
            staging_buffer.write_array(data, Some(offset))?;
            offset += data.len();
        }

        // create image wrap
        use dacite::core::{ImageLayout, ImageTiling, ImageUsageFlags,
                           ComponentMapping};
        let mut image_wrap = ImageWrap::new(
            device, memory, format, component_mapping,
            dds.get_num_mipmap_levels(),
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
            &commander,
            &staging_buffer.inner(),
            dds.get_main_texture_size().unwrap(),
            dds.get_min_mipmap_size_in_bytes()
        )?;

        // transfer layout to ImageLayout::ShaderReadOnlyOptimal
        use dacite::core::{AccessFlags, ImageAspectFlags, OptionalMipLevels,
                           OptionalArrayLayers, ImageSubresourceRange,
                           PipelineStageFlags};
        image_wrap.transition_layout_now(
            device,
            ImageLayout::Undefined, ImageLayout::ShaderReadOnlyOptimal,
            AccessFlags::TRANSFER_WRITE, AccessFlags::SHADER_READ,
            PipelineStageFlags::TRANSFER, PipelineStageFlags::VERTEX_SHADER,
            ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: OptionalArrayLayers::ArrayLayers(num_layers),
            },
            &commander)?;
        // And wait until that completes
        // FIXME - this is a CPU side stall.

        // insert to hashmap
        self.textures.insert(name.to_owned(), image_wrap.clone());

        Ok(image_wrap)
    }

    pub fn load_buffer(
        &mut self,
        device: &Device,
        memory: &mut Memory,
        commander: &Commander,
        staging_buffer: &mut HostVisibleBuffer,
        usage: BufferUsageFlags,
        name: &str)
        -> Result<DeviceLocalBuffer>
    {
        // Check if we already have it
        if let Some(bufref) = self.buffers.get(name) {
            return Ok(bufref.clone());
        }

        let mut path = self.asset_path.clone();

        // All textures under the siege engine are stored raw and
        // compressed with Zstd, and named with the ".raw.zst" extension.
        path.push("buffers");
        path.push(format!("{}.raw.zst", name));
        let f = File::open(path)?;

        // Setup Decompressor
        use zstd::stream::Decoder;
        let mut d = Decoder::new(f)?;

        // Decompress into a device buffer
        let dlb = DeviceLocalBuffer::new_from_reader(
            device, memory, commander,
            &mut d, staging_buffer,
            usage,
            Lifetime::Temporary,
            &*format!("buffer {}", name))?;

        self.buffers.insert(name.to_owned(), dlb.clone());

        Ok(dlb)
    }
}
