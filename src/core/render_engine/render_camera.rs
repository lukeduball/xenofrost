use std::mem::size_of;

use glam::Mat4;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(view_projection_matrix: Mat4) -> Self {
        Self {
            view_proj: view_projection_matrix.to_cols_array_2d(),
        }
    }
}

pub fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None
            }
        ]
    });

    bind_group_layout
}

pub struct RenderCamera {
    uniform_buffer: wgpu::Buffer, 
    pub camera_bind_group: wgpu::BindGroup,
}

impl RenderCamera {
    pub fn new(device: &wgpu::Device, camera_bind_group_layout: &wgpu::BindGroupLayout, name: &str) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor { 
            label: Some((String::from(name) + "_buffer").as_str()), 
            size: size_of::<CameraUniform>() as u64, 
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, 
            mapped_at_creation: false 
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some((String::from(name) + "_bind_group").as_str()),
            layout: camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
        });

        Self {
            uniform_buffer,
            camera_bind_group
        }
    }

    pub fn update_uniform_buffer(&mut self, view_projection_matrix: Mat4, queue: &wgpu::Queue) {
        let camera_uniform = CameraUniform::new(view_projection_matrix);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }
}