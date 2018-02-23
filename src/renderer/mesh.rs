
use errors::*;
use dacite::core::Device;
use siege_mesh::{Mesh, Vertex};
use super::buffer::{DeviceLocalBuffer, HostVisibleBuffer};
use super::memory::{Memory, Lifetime};
use super::commander::Commander;

#[derive(Debug, Clone)]
pub struct VulkanMesh<V: Vertex> {
    pub vertex_buffer: DeviceLocalBuffer<V>,
    pub index_buffer: DeviceLocalBuffer<(u16,u16,u16)>,

    pub num_vertices: u32,
    pub num_indices: u32,

    // TBD: texture images

    // TBD: uniforms buffer
    //   (e.g. maybe we have other per-mesh values like floats and vec4s stored in mesh files)
}

impl<V: Vertex> VulkanMesh<V> {
    pub fn new(device: &Device,
               memory: &mut Memory,
               commander: &Commander,
               staging_buffer: &HostVisibleBuffer<u8>,
               mesh: Mesh<V>,
               name: &str)
               -> Result<VulkanMesh<V>>
    {
        use dacite::core::BufferUsageFlags;

        let vertex_buffer = DeviceLocalBuffer::new_uploaded(
            device, memory, commander,
            staging_buffer, &mesh.vertices,
            BufferUsageFlags::VERTEX_BUFFER,
            Lifetime::Temporary,
            &*format!("{} Vertex Buffer", name))?;

        let index_buffer = DeviceLocalBuffer::new_uploaded(
            device, memory, commander,
            staging_buffer, &mesh.indices,
            BufferUsageFlags::INDEX_BUFFER,
            Lifetime::Temporary,
            &*format!("{} Index Buffer", name))?;

        Ok(VulkanMesh {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            num_vertices: mesh.vertices.len() as u32,
            num_indices: (mesh.indices.len() * 3) as u32 // we group them in 3s
        })
    }
}
