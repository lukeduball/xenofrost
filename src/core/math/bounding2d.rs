use core::f32;

use bytemuck::Zeroable;
use glam::{Mat3, Vec2};

pub struct BoundingBox2d {
    pub center: Vec2,
    pub half_size: Vec2,
    pub rotation: f32
}

struct Intersection2dResult {
    collision: bool,
    normal: Vec2,
    penetration_val: f32
}

impl Intersection2dResult {
    fn new() -> Self {
        Intersection2dResult { collision: false, normal: Vec2::zeroed(), penetration_val: f32::MAX }
    }
}

fn find_separating_axis(box1_vertices: &[Vec2; 4], box1_normals: &[Vec2; 2], box2_vertices: &[Vec2; 4], box2_normals: &[Vec2; 2]) -> Intersection2dResult {
    let mut intersection_result = Intersection2dResult::new();
    
    let box_normals_list = [box1_normals, box2_normals];
    //Loop through the set of normals belonging to each box
    for box_normals in box_normals_list {
        //Perform the check on each normal of a specific box
        for normal in box_normals {
            let mut b1_max = f32::MIN;
            let mut b1_min = f32::MAX;
            for box1_vertex in box1_vertices {
                let projection = normal.dot(*box1_vertex);
                b1_max = b1_max.max(projection);
                b1_min = b1_min.min(projection);
            }
            
            let mut b2_max = f32::MIN;
            let mut b2_min = f32::MAX;
            for box2_vertex in box2_vertices {
                let projection = normal.dot(*box2_vertex);
                b2_max = b2_max.max(projection);
                b2_min = b2_min.min(projection);
            }

            if (b1_min <= b2_max && b1_min >= b2_min) || (b2_min <= b1_max && b2_min >= b1_min) {
                intersection_result.collision = true;
                let penetration1 = f32::abs(b1_min - b2_max);
                let penetration2 = f32::abs(b1_max - b2_min);
                let penetration = penetration1.min(penetration2);
                if penetration < intersection_result.penetration_val {
                    intersection_result.penetration_val = penetration;
                    intersection_result.normal = normal.clone();
                }
            }
            else {
                intersection_result.collision = false;
                return intersection_result
            }
        }
    }

    intersection_result
}

//Only calculate two normals because in a rectangle, the separating axis is the same for normals on opposite sides
fn calculate_normals_of_box2d(box_vertices: &[Vec2; 4]) -> [Vec2; 2] {
    [
        (box_vertices[0] - box_vertices[1]).perp().normalize(),
        (box_vertices[1] - box_vertices[2]).perp().normalize()
    ]
}

impl BoundingBox2d {
    pub fn new(center: Vec2, half_size: Vec2, rotation: f32) -> Self {
        Self {
            center,
            half_size,
            rotation
        }
    }

    fn calculate_vertices(&self) -> [Vec2; 4] {
        let translation_matrix = Mat3::from_translation(self.center);
        let rotation_matrix = Mat3::from_angle(self.rotation.to_radians());
        let combined_matrix = translation_matrix * rotation_matrix;
        
        [
            combined_matrix.transform_point2(Vec2::new(-self.half_size.x, -self.half_size.y)),
            combined_matrix.transform_point2(Vec2::new(-self.half_size.x, self.half_size.y)),
            combined_matrix.transform_point2(Vec2::new(self.half_size.x, self.half_size.y)),
            combined_matrix.transform_point2(Vec2::new(self.half_size.x, -self.half_size.y))
        ]
    }

    pub fn intersects(&self, other_boundingbox2d: &BoundingBox2d) -> bool {
        let vertices = self.calculate_vertices();
        let normals = calculate_normals_of_box2d(&vertices);
        let other_vertices = other_boundingbox2d.calculate_vertices();
        let other_normals = calculate_normals_of_box2d(&other_vertices);

        let separating_axis_result = find_separating_axis(&vertices, &normals, &other_vertices, &other_normals);
        separating_axis_result.collision
    }
}

pub struct Aabb2d {
    pub center: Vec2,
    pub half_size: Vec2,
}

impl Aabb2d {
    pub fn new(center: Vec2, half_size: Vec2) -> Self {
        Self {
            center,
            half_size,
        }
    }

    pub fn intersects(&self, other_aabb2d: &Aabb2d) -> bool {
        let min = self.center - self.half_size;
        let max = self.center + self.half_size;
        let other_aabb2d_min = other_aabb2d.center - other_aabb2d.half_size;
        let other_aabb2d_max = other_aabb2d.center + other_aabb2d.half_size;
        let does_x_overlap = min.x <= other_aabb2d_max.x && max.x >= other_aabb2d_min.x;
        let does_y_overlap = min.y <= other_aabb2d_max.y && max.y >= other_aabb2d_min.y;
        does_x_overlap && does_y_overlap
    }
}

pub struct BoundingCircle {
    center: Vec2,
    radius: f32,
}

impl BoundingCircle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius
        }
    }

    pub fn intersects(&self, other_bounding_circle: &BoundingCircle) -> bool {
        let distance_squared = self.center.distance_squared(other_bounding_circle.center);
        distance_squared <= (self.radius + other_bounding_circle.radius) * (self.radius + other_bounding_circle.radius)
    }
}

#[cfg(test)]
mod bounding2d_tests {
    use glam::Vec2;

    use super::{Aabb2d, BoundingCircle, BoundingBox2d};

    #[test]
    fn boundingbox2d_intersects_test() {
        let box1 = BoundingBox2d::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0), 0.0);
        let box2 = BoundingBox2d::new(Vec2::new(1.0, 1.0), Vec2::new(2.0, 1.0), 45.0);
        let box3 = BoundingBox2d::new(Vec2::new(5.0, 5.0), Vec2::new(1.0, 1.0), 0.0);
        let box4 = BoundingBox2d::new(Vec2::new(-3.0, -2.0), Vec2::new(1.5, 1.0), 90.0);
        let box5 = BoundingBox2d::new(Vec2::new(0.5, 0.5), Vec2::new(2.0, 2.0), 30.0);
        let box6 = BoundingBox2d::new(Vec2::new(-5.0, 3.0), Vec2::new(1.0, 1.0), 60.0);
        let box7 = BoundingBox2d::new(Vec2::new(3.0, -1.0), Vec2::new(1.0, 2.0), 0.0);
        let box8 = BoundingBox2d::new(Vec2::new(4.0, -1.0), Vec2::new(1.0, 2.0), 0.0);
        let box9 = BoundingBox2d::new(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0), 10.0);
        let box10 = BoundingBox2d::new(Vec2::new(-1.5, -1.5), Vec2::new(1.0, 1.0), 10.0);
        let box11 = BoundingBox2d::new(Vec2::new(6.0, 0.0), Vec2::new(2.0, 1.0), 15.0);
        let box12 = BoundingBox2d::new(Vec2::new(7.0, 0.0), Vec2::new(2.0, 1.0), 15.0);
        let box13 = BoundingBox2d::new(Vec2::new(0.0, 0.0), Vec2::new(0.5, 0.5), 90.0);
        let box14 = BoundingBox2d::new(Vec2::new(-6.0, -6.0), Vec2::new(2.0, 2.0), 0.0);
        let box15 = BoundingBox2d::new(Vec2::new(-6.5, -6.5), Vec2::new(1.0, 1.0), 0.0);
        let box16 = BoundingBox2d::new(Vec2::new(1.0, -3.0), Vec2::new(1.0, 1.0), 45.0);
        let box17 = BoundingBox2d::new(Vec2::new(2.0, -3.0), Vec2::new(1.0, 1.0), 45.0);
        let box18 = BoundingBox2d::new(Vec2::new(2.0, 2.0), Vec2::new(0.5, 0.5), 0.0);
        let box19 = BoundingBox2d::new(Vec2::new(2.5, 2.5), Vec2::new(0.5, 0.5), 0.0);
        let box20 = BoundingBox2d::new(Vec2::new(-2.0, 4.0), Vec2::new(1.0, 2.0), 30.0);
        let box21 = BoundingBox2d::new(Vec2::new(-2.5, 4.5), Vec2::new(1.0, 2.0), 30.0);
        let box22 = BoundingBox2d::new(Vec2::new(8.0, 8.0), Vec2::new(2.0, 2.0), 60.0);
        let box23 = BoundingBox2d::new(Vec2::new(10.0, 10.0), Vec2::new(1.0, 1.0), 60.0);
        let box24 = BoundingBox2d::new(Vec2::new(-8.0, -8.0), Vec2::new(3.0, 1.5), 15.0);
        let box25 = BoundingBox2d::new(Vec2::new(-9.0, -9.0), Vec2::new(1.0, 1.0), 0.0);

        assert_eq!(box1.intersects(&box5), true);
        assert_eq!(box1.intersects(&box13), true);
        assert_eq!(box5.intersects(&box13), true);
        assert_eq!(box7.intersects(&box8), true);
        assert_eq!(box9.intersects(&box10), true);
        assert_eq!(box11.intersects(&box12), true);
        assert_eq!(box14.intersects(&box15), true);
        assert_eq!(box16.intersects(&box17), true);
        assert_eq!(box18.intersects(&box19), true);
        assert_eq!(box20.intersects(&box21), true);
        assert_eq!(box24.intersects(&box25), true);
        assert_eq!(box2.intersects(&box5), true);
        assert_eq!(box5.intersects(&box10), true);
        assert_eq!(box9.intersects(&box13), true);
        assert_eq!(box12.intersects(&box11), true);
        assert_eq!(box17.intersects(&box7), true);
        assert_eq!(box1.intersects(&box2), true);
        assert_eq!(box18.intersects(&box2), true);
        assert_eq!(box4.intersects(&box10), true);
        assert_eq!(box7.intersects(&box16), true);
        
        assert_eq!(box2.intersects(&box4), false);        
        assert_eq!(box6.intersects(&box21), false);
        assert_eq!(box14.intersects(&box10), false);
        assert_eq!(box3.intersects(&box2), false);
        assert_eq!(box6.intersects(&box20), false);
        assert_eq!(box19.intersects(&box5), false);
        assert_eq!(box1.intersects(&box22), false);
        assert_eq!(box2.intersects(&box23), false);
        assert_eq!(box3.intersects(&box24), false);
        assert_eq!(box4.intersects(&box6), false);
        assert_eq!(box6.intersects(&box16), false);
        assert_eq!(box7.intersects(&box20), false);
        assert_eq!(box8.intersects(&box25), false);
        assert_eq!(box9.intersects(&box23), false);
        assert_eq!(box10.intersects(&box18), false);
        assert_eq!(box11.intersects(&box20), false);
        assert_eq!(box12.intersects(&box6), false);
        assert_eq!(box13.intersects(&box21), false);
        assert_eq!(box14.intersects(&box5), false);
        assert_eq!(box15.intersects(&box2), false);
        assert_eq!(box16.intersects(&box21), false);
        assert_eq!(box17.intersects(&box6), false);
        assert_eq!(box18.intersects(&box24), false);
        assert_eq!(box19.intersects(&box4), false);
        assert_eq!(box20.intersects(&box1), false);
        assert_eq!(box21.intersects(&box3), false);
        assert_eq!(box22.intersects(&box9), false);
        assert_eq!(box23.intersects(&box5), false);
        assert_eq!(box24.intersects(&box8), false);
        assert_eq!(box25.intersects(&box11), false);
    }

    #[test]
    fn aabb2d_intersects_test() {
        let aabb2d = Aabb2d::new(Vec2::new(0.0, 0.0), Vec2::splat(1.0));
        assert!(aabb2d.intersects(&aabb2d));

        // x intersection tests
        let aabb2d_2 = Aabb2d::new(Vec2::new(2.0, 0.0), Vec2::splat(1.0));
        assert!(aabb2d.intersects(&aabb2d_2));
        let aabb2d_3 = Aabb2d::new(Vec2::new(2.1, 0.0), Vec2::splat(1.0));
        assert!(!aabb2d.intersects(&aabb2d_3));
        let aabb2d_4 = Aabb2d::new(Vec2::new(-2.0, 0.0), Vec2::splat(1.0));
        assert!(aabb2d.intersects(&aabb2d_4));
        let aabb2d_5 = Aabb2d::new(Vec2::new(-2.1, 0.0), Vec2::splat(1.0));
        assert!(!aabb2d.intersects(&aabb2d_5));
        
        // y intersection tests
        let aabb2d_6 = Aabb2d::new(Vec2::new(0.0, 2.0), Vec2::splat(1.0));
        assert!(aabb2d.intersects(&aabb2d_6));
        let aabb2d_7 = Aabb2d::new(Vec2::new(0.0, 2.1), Vec2::splat(1.0));
        assert!(!aabb2d.intersects(&aabb2d_7));
        let aabb2d_8 = Aabb2d::new(Vec2::new(0.0, -2.0), Vec2::splat(1.0));
        assert!(aabb2d.intersects(&aabb2d_8));
        let aabb2d_9 = Aabb2d::new(Vec2::new(0.0, -2.1), Vec2::splat(1.0));
        assert!(!aabb2d.intersects(&aabb2d_9));

        let aabb2d_10 = Aabb2d::new(Vec2::new(0.9, -0.67), Vec2::new(0.25, 0.9));
        assert!(aabb2d.intersects(&aabb2d_10));
        assert!(!aabb2d_4.intersects(&aabb2d_10));
    }

    #[test]
    fn bounding_circle_intersects_test() {
        let bounding_circle = BoundingCircle::new(Vec2::new(0.0, 0.0), 1.0);
        assert!(bounding_circle.intersects(&bounding_circle));

        // x tangential intersections
        let bounding_circle_1 = BoundingCircle::new(Vec2::new(2.0, 0.0), 1.0);
        assert!(bounding_circle.intersects(&bounding_circle_1));
        let bounding_circle_2 = BoundingCircle::new(Vec2::new(2.1, 0.0), 1.0);
        assert!(!bounding_circle.intersects(&bounding_circle_2));
        let bounding_circle_3 = BoundingCircle::new(Vec2::new(-2.0, 0.0), 1.0);
        assert!(bounding_circle.intersects(&bounding_circle_3));
        let bounding_circle_4 = BoundingCircle::new(Vec2::new(-2.1, 0.0), 1.0);
        assert!(!bounding_circle.intersects(&bounding_circle_4));

        //y tangential intersections
        let bounding_circle_5 = BoundingCircle::new(Vec2::new(0.0, 2.0), 1.0);
        assert!(bounding_circle.intersects(&bounding_circle_5));
        let bounding_circle_6 = BoundingCircle::new(Vec2::new(0.0, 2.1), 1.0);
        assert!(!bounding_circle.intersects(&bounding_circle_6));
        let bounding_circle_7 = BoundingCircle::new(Vec2::new(0.0, -2.0), 1.0);
        assert!(bounding_circle.intersects(&bounding_circle_7));
        let bounding_circle_8 = BoundingCircle::new(Vec2::new(0.0, -2.1), 1.0);
        assert!(!bounding_circle.intersects(&bounding_circle_8));

        // tangential
        let bounding_circle_9 = BoundingCircle::new(Vec2::new(1.41, -1.41), 1.0);
        assert!(bounding_circle.intersects(&bounding_circle_9));

        let bounding_circle_10 = BoundingCircle::new(Vec2::new(-0.5, 0.24), 0.743);
        assert!(bounding_circle.intersects(&bounding_circle_10));
        assert!(!bounding_circle_7.intersects(&bounding_circle_10));
    }
}