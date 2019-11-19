
use error::*;
use dacite::core::Device;
use siege_math::Point3;
use siege_mesh::{Mesh, Vertex};
use super::buffer::{DeviceLocalBuffer, HostVisibleBuffer};
use super::memory::{Memory, Lifetime};
use super::commander::Commander;

#[derive(Debug, Clone)]
pub struct VulkanMesh {
    pub vertex_buffer: DeviceLocalBuffer,
    pub index_buffer: DeviceLocalBuffer,

    pub num_vertices: u32,
    pub num_indices: u32,

    pub bounding_sphere: Option<(Point3<f32>, f32)>,
    pub bounding_cuboid: Option<[Point3<f32>; 8]>,

    // TBD: texture images

    // TBD: uniforms buffer
    //   (e.g. maybe we have other per-mesh values like floats and vec4s stored in mesh files)
}

impl VulkanMesh {
    pub fn new<V: Vertex>(device: &Device,
                          memory: &mut Memory,
                          commander: &Commander,
                          staging_buffer: &mut HostVisibleBuffer,
                          mesh: Mesh<V>,
                          name: &str)
               -> Result<VulkanMesh, Error>
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
            num_indices: (mesh.indices.len() * 3) as u32, // we group them in 3s
            bounding_sphere: mesh.bounding_sphere.clone(),
            bounding_cuboid: mesh.bounding_cuboid.clone(),
        })
    }
}
