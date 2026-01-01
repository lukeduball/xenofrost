use glam::{IVec2, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

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

pub struct Camera2d {
    pub position: Vec3,
    pub projection: CameraProjection,
    pub view_projection_matrix: Mat4,
}

impl Camera2d {
    pub fn new(position: Vec3, projection: CameraProjection) -> Self {
        let view_projection_matrix = generate_view_projection_matrix(position, Vec3::new(0.0, 0.0, 1.0), &projection);

        Self {
            position,
            projection,
            view_projection_matrix
        }
    }

    pub fn convert_screen_space_to_world_space(&self, screen_pixels: IVec2, window_size: IVec2) -> Vec3 {
        convert_screen_space_to_world_space_util(self.view_projection_matrix, screen_pixels, window_size)
    }

    pub fn update_view_projection_matrix(&mut self) {
        self.view_projection_matrix = generate_view_projection_matrix(self.position, Vec3::new(0.0, 0.0, 1.0), &self.projection);
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        match &mut self.projection {
            CameraProjection::Perspective(perspective_projection) => perspective_projection.aspect_ratio = aspect_ratio,
            CameraProjection::Orthographic(orthographic_projection) => orthographic_projection.aspect_ratio = aspect_ratio,
        }

        self.update_view_projection_matrix();
    }
}

fn generate_view_projection_matrix(position: Vec3, direction: Vec3, projection: &CameraProjection) -> Mat4 {
    let view_proj = match projection {
        CameraProjection::Perspective(perspective_projection) => perspective_projection.build_view_projection_matrix(position, direction),
        CameraProjection::Orthographic(orthographic_projection) => orthographic_projection.build_view_projection_matrix(position, direction),
    };

    view_proj
}

pub fn convert_screen_space_to_world_space_util(view_projection_matrix: Mat4, screen_pixels: IVec2, window_size: IVec2) -> Vec3 {
    let normalized_screen_coords = Vec2::splat(2.0) * screen_pixels.as_vec2() / window_size.as_vec2() - Vec2::splat(1.0);

    let screen_pos = Vec4::new(normalized_screen_coords.x, -normalized_screen_coords.y, -1.0, 1.0);

    let world_pos = view_projection_matrix.inverse() * screen_pos;
    world_pos.xyz()
}