use std::ops::{Deref, DerefMut};

use wgpu::{BufferUsages, util::DeviceExt};

pub struct Buffer {
    label: String,
    usage: wgpu::BufferUsages,
    buffer: wgpu::Buffer
}

impl Deref for Buffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Buffer {
    pub fn empty_buffer(device: &wgpu::Device, label: String, size: u64, usage: wgpu::BufferUsages) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor { 
            label: Some(label.as_str()), 
            size, 
            usage, 
            mapped_at_creation: false 
        });

        Self { label, usage, buffer }
    }

    pub fn create_buffer_during_init(device: &wgpu::Device, label: String, contents: &[u8], usage: wgpu::BufferUsages) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            contents,
            usage,
        });

        Self { label, usage, buffer }
    }

    pub fn update_buffer_data(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, contents: &[u8]) {
        if self.buffer.size() != (contents.len() * size_of::<u8>()) as u64 {
            self.buffer.destroy();
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(self.label.as_str()),
                contents,
                usage: self.usage
            });
        } else {
            queue.write_buffer(&self.buffer, 0, contents);
        }
    }
}

pub struct VecBuffer<T: bytemuck::Zeroable + bytemuck::Pod> {
    contents: Vec<T>,
    buffer: Buffer,
}

impl<'a, T: bytemuck::Zeroable + bytemuck::Pod> Deref for VecBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<'a, T: bytemuck::Zeroable + bytemuck::Pod> DerefMut for VecBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents
    }
}

impl<'a, T: bytemuck::Zeroable + bytemuck::Pod> VecBuffer<T> {
    pub fn new(device: &wgpu::Device, label: &str, usage: BufferUsages) -> Self {
        Self { 
            contents: Vec::new(), 
            buffer: Buffer::empty_buffer(device, label.to_string(), 1, usage) 
        }
    }

    pub fn update_buffer_data(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.buffer.update_buffer_data(device, queue, bytemuck::cast_slice(&self.contents));
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    } 
}