use std::mem::size_of;

use glam::{IVec2, Vec2, Vec4, Vec4Swizzles};
use glam::{Mat4, Vec3};

use crate::core::world::component::Component;
use crate::core::world::resource::ResourceHandle;
use crate::core::world::resource::Resource;
use crate::core::world::World;
use crate::core::world::query_resource;

use super::RenderEngine;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

#[derive(Resource)]
pub struct CameraBindGroupLayout {
    pub bind_group_layout: wgpu::BindGroupLayout
}

impl CameraBindGroupLayout {
    pub fn new(render_engine: &ResourceHandle<RenderEngine>) -> Self {
        Self { 
            bind_group_layout: render_engine.data().device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None, 
                        },
                        count: None,
                    }
                ],
                label: Some("camera_bind_group_layout"),
            }) 
        }
    }
}

const UP_DIRECTION_VECTOR: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub struct PerspectiveProjection {
    field_of_view: f32,
    aspect_ratio: f32,
    near_clip: f32,
    far_clip: f32
}

impl PerspectiveProjection {
    pub fn build_view_projection_matrix(&self, position: Vec3, direction: Vec3) -> Mat4 {
        let view = Mat4::look_to_lh(position, direction, UP_DIRECTION_VECTOR);
        let projection = Mat4::perspective_lh(self.field_of_view.to_radians(), self.aspect_ratio, self.near_clip, self.far_clip);

        projection * view
    }
}

pub struct OrthographicProjection {
    pub width: f32,
    pub height: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub aspect_ratio: f32,
}

impl OrthographicProjection {
    pub fn build_view_projection_matrix(&self, position: Vec3, direction: Vec3) -> Mat4 {
        let view = Mat4::look_to_lh(position, direction, UP_DIRECTION_VECTOR);

        let half_width = (self.width * self.aspect_ratio) / 2.0;
        let half_height = self.height / 2.0;
        let projection = Mat4::orthographic_lh(-half_width, half_width, -half_height, half_height, self.near_clip, self.far_clip);

        projection * view
    }
}

pub enum CameraProjection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection)
}

#[derive(Component)]
pub struct Camera {
    pub projection: CameraProjection,
    pub uniform_buffer: wgpu::Buffer, 
    pub camera_bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(name: &str, projection: CameraProjection, world: &mut World) -> Self {
        let render_engine = query_resource!(world, RenderEngine).unwrap();
        let camera_bind_group_layout = query_resource!(world, CameraBindGroupLayout).unwrap();

        let uniform_buffer = render_engine.data().device.create_buffer(&wgpu::BufferDescriptor { 
            label: Some((String::from(name) + "_buffer").as_str()), 
            size: size_of::<CameraUniform>() as u64, 
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, 
            mapped_at_creation: false 
        });

        let camera_bind_group = render_engine.data().device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some((String::from(name) + "_bind_group").as_str()),
            layout: &camera_bind_group_layout.data().bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
        });

        Self {
            projection,
            uniform_buffer,
            camera_bind_group
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        match &mut self.projection {
            CameraProjection::Perspective(perspective_projection) => perspective_projection.aspect_ratio = aspect_ratio,
            CameraProjection::Orthographic(orthographic_projection) => orthographic_projection.aspect_ratio = aspect_ratio,
        }
    }

    pub fn update_uniform_buffer(&mut self, position: Vec3, direction: Vec3, queue: &wgpu::Queue) {
        let mut camera_uniform = CameraUniform::new();

        match &mut self.projection {
            CameraProjection::Perspective(perspective_projection) => camera_uniform.view_proj = perspective_projection.build_view_projection_matrix(position, direction).to_cols_array_2d(),
            CameraProjection::Orthographic(orthographic_projection) => camera_uniform.view_proj = orthographic_projection.build_view_projection_matrix(position, direction).to_cols_array_2d(),
        }

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    pub fn convert_screen_space_to_camera_space(&self, position: Vec3, direction: Vec3, screen_pixels: IVec2, window_size: IVec2) -> Vec3 {
        let normalized_screen_coords = Vec2::splat(2.0) * screen_pixels.as_vec2() / window_size.as_vec2() - Vec2::splat(1.0);

        let screen_pos = Vec4::new(normalized_screen_coords.x, -normalized_screen_coords.y, -1.0, 1.0);
        
        let view_proj = match &self.projection {
            CameraProjection::Perspective(perspective_projection) => perspective_projection.build_view_projection_matrix(position, direction),
            CameraProjection::Orthographic(orthographic_projection) => orthographic_projection.build_view_projection_matrix(position, direction),
        };

        let world_pos = view_proj.inverse() * screen_pos;
        world_pos.xyz()
    }
}