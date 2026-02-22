use std::{cmp::min, rc::Rc};

use glam::Vec2;

use crate::core::{math::{Transform2d, bounding2d::Polygon2d}, utilities::{self, Timer}};

pub mod camera;

pub struct AnimationFrame2d {
    time_to_next_frame: u32,
    texture_coordinates: Vec2,
}

impl AnimationFrame2d {
    pub fn new_from_frames(time_to_next_frame: u32, texture_coordinates: Vec2) -> Self {
        Self {
            time_to_next_frame,
            texture_coordinates
        }
    }

    pub fn new_from_seconds(time_to_next_frame_sec: f32, texture_coordinates: Vec2) -> Self {
        let time_to_next_frame = utilities::convert_seconds_to_frames(time_to_next_frame_sec);
        Self::new_from_frames(time_to_next_frame, texture_coordinates)
    }

    pub fn get_texture_coords(&self) -> Vec2 {
        self.texture_coordinates
    }
}

pub struct Animation2d {
    animation_frame_list: Vec<AnimationFrame2d>
}

impl Animation2d {
    pub fn new() -> Self {
        Self {
            animation_frame_list: Vec::new()
        }
    }

    pub fn add_animation_frame(&mut self, animation_frame: AnimationFrame2d) {
        self.animation_frame_list.push(animation_frame);
    }
}

pub struct AnimationObject2d {
    pub transform2d: Transform2d,
    current_index: u32,
    animation_timer: Timer,
    animation: Rc<Animation2d>
}

impl AnimationObject2d {
    pub fn new(transform2d: Transform2d, animation: Rc<Animation2d>) -> Self {
        let frames = animation.animation_frame_list[0].time_to_next_frame;
        let animation_timer = Timer::create_timer_from_update_frames(frames);
        Self {
            transform2d,
            current_index: 0,
            animation_timer,
            animation
        }
    }

    pub fn get_texture_coords_for_current_frame(&self) -> Vec2 {
        let index = min(self.animation.animation_frame_list.len() - 1, self.current_index as usize);
        self.animation.animation_frame_list[index].get_texture_coords()
    }

    pub fn run_animation(&mut self) {
        if !self.is_animation_complete() {
            if self.animation_timer.is_timer_expired() {
                self.current_index += 1;
                
                if self.current_index < self.animation.animation_frame_list.len() as u32 {
                    let next_timer_time_frames = self.animation.animation_frame_list[self.current_index as usize].time_to_next_frame;
                    self.animation_timer.set_expire_time_from_frames(next_timer_time_frames);
                    self.animation_timer.initialize_timer();
                }
            }
            else {
                self.animation_timer.run();
            }
        }
    }

    pub fn is_animation_complete(&self) -> bool {
        self.current_index >= self.animation.animation_frame_list.len() as u32
    }
}

pub trait WorldObject2d {
    fn get_transform2d(&self) -> &Transform2d;

    fn translate(&mut self, translation: Vec2);
    fn set_translation(&mut self, translation: Vec2);
    fn rotate(&mut self, rotation: f32);
    fn set_rotation(&mut self, rotation: f32);
    fn scale(&mut self, scale_factor: Vec2);
    fn set_scale(&mut self, scale_factor: Vec2);
}

pub trait WorldCollisionObject2d {
    fn get_collider(&self) -> &Polygon2d;
}