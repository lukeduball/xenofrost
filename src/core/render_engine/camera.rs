use glam::{Mat4, Vec3};
use xenofrost_macros::Component;

use crate::core::world::component::Component;
use crate::core::input_manager::InputManager;

pub struct OrthoCamera {
    eye: Vec3,
    direction: Vec3,
    up: Vec3,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
}

impl OrthoCamera {
    pub fn new(eye: Vec3, direction: Vec3, up: Vec3, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> OrthoCamera {
        OrthoCamera {
            eye,
            direction,
            up,
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    pub fn resize(&mut self, aspect_ratio: f32) {
        let width = (self.top * 2.0) * aspect_ratio;
        self.left = width / -2.0;
        self.right = width / 2.0;
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_to_lh(self.eye, self.direction, self.up);

        let projection = Mat4::orthographic_lh(self.left, self.right, self.bottom, self.top, self.near, self.far);

        return projection * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &OrthoCamera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
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

    pub fn update_camera(&self, camera: &mut OrthoCamera, input_manager: &InputManager) {
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
    }
}

struct PerspectiveProjection {
    field_of_view: f32,
    aspect_ratio: f32,
    near_clip: f32,
    far_clip: f32
}

struct OrthographicProjection {
    width: f32,
    height: f32,
    near_clip: f32,
    far_clip: f32
}

enum CameraProjection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection)
}

#[derive(Component)]
struct Camera {
    projection: CameraProjection,
    camera_uniform: CameraUniform,
    camera_bind_group: wgpu::BindGroup,
}