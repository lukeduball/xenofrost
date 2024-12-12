use wgpu::{util::DeviceExt, BufferUsages};

pub struct Material {
    name: String,
    color_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub fn create_color_material(device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, color: [f32; 3], name: String) -> Material {
    let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("Material {name} Color Buffer")),
        contents: bytemuck::cast_slice(&color),
        usage: BufferUsages::UNIFORM
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("Material {name} Bind Group")),
        layout: layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }
        ],
    });
    
    Material {
        name,
        color_buffer,
        bind_group
    }
}