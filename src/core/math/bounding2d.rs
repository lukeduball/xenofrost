use core::f32;

use bytemuck::Zeroable;
use glam::{Mat3, Vec2};

pub enum CollisionPrimitive2d {
    Obb2d(BoundingBox2d),
    Aabb2d(Aabb2d),
    Polygon2d(),
    Circle(BoundingCircle)
}

impl CollisionPrimitive2d {
    pub fn intersects(&self, other_primitive: &CollisionPrimitive2d) -> Intersection2dResult {
        match self {
            Self::Obb2d(obb2d) => {
                match other_primitive {
                    Self::Obb2d(other_obb2d) => get_obb2d_with_obb2d_intersection(obb2d, other_obb2d),
                    Self::Aabb2d(other_aabb2d) => todo!(), 
                    Self::Polygon2d() => todo!(),
                    Self::Circle(other_bounding_circle) => todo!()
                }
            },
            Self::Aabb2d(aabb2d) => {
                match other_primitive {
                    Self::Obb2d(other_obb2d) => todo!(),
                    Self::Aabb2d(other_aabb2d) => get_aabb2d_with_aabb2d_intersection(aabb2d, other_aabb2d), 
                    Self::Polygon2d() => todo!(),
                    Self::Circle(other_bounding_circle) => todo!()
                }
            },
            Self::Polygon2d() => {
                todo!()
            },
            Self::Circle(bounding_circle) => {
                match other_primitive {
                    Self::Obb2d(other_obb2d) => todo!(),
                    Self::Aabb2d(other_aabb2d) => todo!(), 
                    Self::Polygon2d() => todo!(),
                    Self::Circle(other_bounding_circle) => get_circle_with_circle_intersection(bounding_circle, other_bounding_circle)
                }
            }
        }
    }
}

pub struct BoundingBox2d {
    pub center: Vec2,
    pub half_size: Vec2,
    pub rotation: f32
}

pub struct Intersection2dResult {
    collision: bool,
    normal: Vec2,
    penetration_val: f32
}

impl Intersection2dResult {
    fn new() -> Self {
        Intersection2dResult { collision: false, normal: Vec2::zeroed(), penetration_val: f32::MAX }
    }
}

fn get_circle_with_circle_intersection(circle1: &BoundingCircle, circle2: &BoundingCircle) -> Intersection2dResult {
    let mut result = Intersection2dResult::new();
    
    let distance_squared = circle1.center.distance_squared(circle2.center);
    if distance_squared <= (circle1.radius + circle2.radius) * (circle1.radius + circle2.radius) {
        result.collision = true;
        result.normal = (circle2.center - circle1.center).normalize();
        result.penetration_val = distance_squared.sqrt();
    }

    result
}

fn get_obb2d_with_obb2d_intersection(obb2d_1: &BoundingBox2d, obb2d_2: &BoundingBox2d) -> Intersection2dResult {
    let vertices = obb2d_1.calculate_vertices();
    let normals = calculate_normals_of_box2d(&vertices);
    let other_vertices = obb2d_2.calculate_vertices();
    let other_normals = calculate_normals_of_box2d(&other_vertices);

    let separating_axis_result = find_separating_axis(&vertices, &normals, &other_vertices, &other_normals);
    separating_axis_result
}

fn get_aabb2d_with_aabb2d_intersection(aabb2d_1: &Aabb2d, aabb2d_2: &Aabb2d) -> Intersection2dResult {
    let mut result = Intersection2dResult::new();

    let min = aabb2d_1.center - aabb2d_1.half_size;
    let max = aabb2d_1.center + aabb2d_1.half_size;
    let other_aabb2d_min = aabb2d_2.center - aabb2d_2.half_size;
    let other_aabb2d_max = aabb2d_2.center + aabb2d_2.half_size;
    let does_x_overlap = min.x <= other_aabb2d_max.x && max.x >= other_aabb2d_min.x;
    let does_y_overlap = min.y <= other_aabb2d_max.y && max.y >= other_aabb2d_min.y;
    if does_x_overlap && does_y_overlap {
        result.collision = true;
        //TODO add resolution vectors
    }

    result
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
}

#[cfg(test)]
mod bounding2d_tests {
    use glam::Vec2;

    use crate::core::math::bounding2d::{get_aabb2d_with_aabb2d_intersection, get_circle_with_circle_intersection, get_obb2d_with_obb2d_intersection};

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

        assert_eq!(get_obb2d_with_obb2d_intersection(&box1, &box5).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box1, &box13).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box5, &box13).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box7, &box8).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box9, &box10).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box11, &box12).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box14, &box15).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box16, &box17).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box18, &box19).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box20, &box21).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box24, &box25).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box2, &box5).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box5, &box10).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box9, &box13).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box12, &box11).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box17, &box7).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box1, &box2).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box18, &box2).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box4, &box10).collision, true);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box7, &box16).collision, true);
        
        assert_eq!(get_obb2d_with_obb2d_intersection(&box2, &box4).collision, false);        
        assert_eq!(get_obb2d_with_obb2d_intersection(&box6, &box21).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box14, &box10).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box3, &box2).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box6, &box20).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box19, &box5).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box1, &box22).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box2, &box23).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box3, &box24).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box4, &box6).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box6, &box16).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box7, &box20).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box8, &box25).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box9, &box23).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box10, &box18).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box11, &box20).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box12, &box6).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box13, &box21).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box14, &box5).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box15, &box2).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box16, &box21).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box17, &box6).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box18, &box24).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box19, &box4).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box20, &box1).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box21, &box3).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box22, &box9).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box23, &box5).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box24, &box8).collision, false);
        assert_eq!(get_obb2d_with_obb2d_intersection(&box25, &box11).collision, false);
    }

    #[test]
    fn aabb2d_intersects_test() {
        let aabb2d = Aabb2d::new(Vec2::new(0.0, 0.0), Vec2::splat(1.0));
        assert!(get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d).collision);

        // x intersection tests
        let aabb2d_2 = Aabb2d::new(Vec2::new(2.0, 0.0), Vec2::splat(1.0));
        assert!(get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_2).collision);
        let aabb2d_3 = Aabb2d::new(Vec2::new(2.1, 0.0), Vec2::splat(1.0));
        assert!(!get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_3).collision);
        let aabb2d_4 = Aabb2d::new(Vec2::new(-2.0, 0.0), Vec2::splat(1.0));
        assert!(get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_4).collision);
        let aabb2d_5 = Aabb2d::new(Vec2::new(-2.1, 0.0), Vec2::splat(1.0));
        assert!(!get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_5).collision);
        
        // y intersection tests
        let aabb2d_6 = Aabb2d::new(Vec2::new(0.0, 2.0), Vec2::splat(1.0));
        assert!(get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_6).collision);
        let aabb2d_7 = Aabb2d::new(Vec2::new(0.0, 2.1), Vec2::splat(1.0));
        assert!(!get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_7).collision);
        let aabb2d_8 = Aabb2d::new(Vec2::new(0.0, -2.0), Vec2::splat(1.0));
        assert!(get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_8).collision);
        let aabb2d_9 = Aabb2d::new(Vec2::new(0.0, -2.1), Vec2::splat(1.0));
        assert!(!get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_9).collision);

        let aabb2d_10 = Aabb2d::new(Vec2::new(0.9, -0.67), Vec2::new(0.25, 0.9));
        assert!(get_aabb2d_with_aabb2d_intersection(&aabb2d, &aabb2d_10).collision);
        assert!(!get_aabb2d_with_aabb2d_intersection(&aabb2d_4, &aabb2d_10).collision);
    }

    #[test]
    fn bounding_circle_intersects_test() {
        let bounding_circle = BoundingCircle::new(Vec2::new(0.0, 0.0), 1.0);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle).collision);

        // x tangential intersections
        let bounding_circle_1 = BoundingCircle::new(Vec2::new(2.0, 0.0), 1.0);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_1).collision);
        let bounding_circle_2 = BoundingCircle::new(Vec2::new(2.1, 0.0), 1.0);
        assert!(!get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_2).collision);
        let bounding_circle_3 = BoundingCircle::new(Vec2::new(-2.0, 0.0), 1.0);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_3).collision);
        let bounding_circle_4 = BoundingCircle::new(Vec2::new(-2.1, 0.0), 1.0);
        assert!(!get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_4).collision);

        //y tangential intersections
        let bounding_circle_5 = BoundingCircle::new(Vec2::new(0.0, 2.0), 1.0);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_5).collision);
        let bounding_circle_6 = BoundingCircle::new(Vec2::new(0.0, 2.1), 1.0);
        assert!(!get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_6).collision);
        let bounding_circle_7 = BoundingCircle::new(Vec2::new(0.0, -2.0), 1.0);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_7).collision);
        let bounding_circle_8 = BoundingCircle::new(Vec2::new(0.0, -2.1), 1.0);
        assert!(!get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_8).collision);

        // tangential
        let bounding_circle_9 = BoundingCircle::new(Vec2::new(1.41, -1.41), 1.0);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_9).collision);

        let bounding_circle_10 = BoundingCircle::new(Vec2::new(-0.5, 0.24), 0.743);
        assert!(get_circle_with_circle_intersection(&bounding_circle, &bounding_circle_10).collision);
        assert!(!get_circle_with_circle_intersection(&bounding_circle_7, &bounding_circle_10).collision);
    }
}