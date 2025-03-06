use wgpu::util::DeviceExt;
use xenofrost_macros::Resource;

use crate::core::world::resource::Resource;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2]
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

#[derive(Resource)]
pub struct QuadMesh {
    pub mesh: Mesh
}

impl QuadMesh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            mesh: create_quad_mesh(device)
        }
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

const QUAD_VERTICES: &[ModelVertex] = &[
    ModelVertex {position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0]},
    ModelVertex {position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0]},
    ModelVertex {position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0]},
    ModelVertex {position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0]},
];

const QUAD_INDICES: &[u16] = &[
    0, 1, 2,
    1, 3, 2
];

pub fn create_quad_mesh(device: &wgpu::Device) -> Mesh {

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Quad Vertex Buffer"),
        contents: bytemuck::cast_slice(QUAD_VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Quad Index Buffer"),
        contents: bytemuck::cast_slice(QUAD_INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    Mesh {
        name: String::from("Quad Mesh"),
        vertex_buffer,
        index_buffer,
        num_elements: QUAD_INDICES.len() as u32,
    }
}