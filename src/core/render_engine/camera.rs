use std::mem::size_of;

use glam::{Mat4, Vec3};
use xenofrost_macros::get_resource_id;
use xenofrost_macros::{Component, Resource};

use crate::core::world::component::Component;
use crate::core::input_manager::InputManager;
use crate::core::world::resource::ResourceHandle;
use crate::core::world::resource::Resource;
use crate::core::world::World;

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

pub struct CameraController {
    speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed
        }
    }

    /*pub fn update_camera(&self, camera: &mut OrthoCamera, input_manager: &InputManager) {
        let left_key_state = input_manager.get_key_state("left").unwrap();
        let right_key_state = input_manager.get_key_state("right").unwrap();
        let up_key_state = input_manager.get_key_state("up").unwrap();
        let down_key_state = input_manager.get_key_state("down").unwrap();

        if left_key_state.get_is_down() {
            camera.eye.x -= self.speed;
        }
        if right_key_state.get_is_down() {
            camera.eye.x += self.speed;
        }
        if up_key_state.get_is_down() {
            camera.eye.y += self.speed;
        }
        if down_key_state.get_is_down() {
            camera.eye.y -= self.speed;
        }
    }*/
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

pub struct PerspectiveProjection {
    field_of_view: f32,
    aspect_ratio: f32,
    near_clip: f32,
    far_clip: f32
}

pub struct OrthographicProjection {
    pub width: f32,
    pub height: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub aspect_ratio: f32,
}

const UP_DIRECTION_VECTOR: Vec3 = Vec3::new(0.0, 1.0, 0.0);

impl OrthographicProjection {
    pub fn build_view_projection_matrix(&self, position: Vec3, direction: Vec3) -> Mat4 {
        let view = Mat4::look_to_lh(position, direction, UP_DIRECTION_VECTOR);

        let half_width = (self.width * self.aspect_ratio) / 2.0;
        let half_height = self.height / 2.0;
        let projection = Mat4::orthographic_lh(-half_width, half_width, -half_height, half_height, self.near_clip, self.far_clip);

        return projection * view;
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
        let render_engine = world.query_resource::<RenderEngine>(get_resource_id!(RenderEngine)).unwrap();
        let camera_bind_group_layout = world.query_resource::<CameraBindGroupLayout>(get_resource_id!(CameraBindGroupLayout)).unwrap();

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
            CameraProjection::Perspective(perspective_projection) => todo!(),
            CameraProjection::Orthographic(orthographic_projection) => camera_uniform.view_proj = orthographic_projection.build_view_projection_matrix(position, direction).to_cols_array_2d(),
        }

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }
}