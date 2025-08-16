use core::f32;

use bytemuck::Zeroable;
use glam::{Mat3, Vec2};

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
pub struct Obb2d {
    pub center: Vec2,
    pub half_size: Vec2,
    pub rotation: f32
}

impl Obb2d {
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

    fn get_intersection_result(&self, other_obb2d: &Obb2d) -> Intersection2dResult {
        let vertices = self.calculate_vertices();
        let normals = calculate_normals_of_box2d(&vertices);
        let other_vertices = other_obb2d.calculate_vertices();
        let other_normals = calculate_normals_of_box2d(&other_vertices);

        let separating_axis_result = find_separating_axis(&vertices, &normals, &other_vertices, &other_normals);
        separating_axis_result
    }
}

#[cfg(test)]
mod bounding2d_tests {
    use glam::Vec2;

    use super::{Obb2d};

    #[test]
    fn obb2d_intersects_test() {
        let box1 = Obb2d::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0), 0.0);
        let box2 = Obb2d::new(Vec2::new(1.0, 1.0), Vec2::new(2.0, 1.0), 45.0);
        let box3 = Obb2d::new(Vec2::new(5.0, 5.0), Vec2::new(1.0, 1.0), 0.0);
        let box4 = Obb2d::new(Vec2::new(-3.0, -2.0), Vec2::new(1.5, 1.0), 90.0);
        let box5 = Obb2d::new(Vec2::new(0.5, 0.5), Vec2::new(2.0, 2.0), 30.0);
        let box6 = Obb2d::new(Vec2::new(-5.0, 3.0), Vec2::new(1.0, 1.0), 60.0);
        let box7 = Obb2d::new(Vec2::new(3.0, -1.0), Vec2::new(1.0, 2.0), 0.0);
        let box8 = Obb2d::new(Vec2::new(4.0, -1.0), Vec2::new(1.0, 2.0), 0.0);
        let box9 = Obb2d::new(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0), 10.0);
        let box10 = Obb2d::new(Vec2::new(-1.5, -1.5), Vec2::new(1.0, 1.0), 10.0);
        let box11 = Obb2d::new(Vec2::new(6.0, 0.0), Vec2::new(2.0, 1.0), 15.0);
        let box12 = Obb2d::new(Vec2::new(7.0, 0.0), Vec2::new(2.0, 1.0), 15.0);
        let box13 = Obb2d::new(Vec2::new(0.0, 0.0), Vec2::new(0.5, 0.5), 90.0);
        let box14 = Obb2d::new(Vec2::new(-6.0, -6.0), Vec2::new(2.0, 2.0), 0.0);
        let box15 = Obb2d::new(Vec2::new(-6.5, -6.5), Vec2::new(1.0, 1.0), 0.0);
        let box16 = Obb2d::new(Vec2::new(1.0, -3.0), Vec2::new(1.0, 1.0), 45.0);
        let box17 = Obb2d::new(Vec2::new(2.0, -3.0), Vec2::new(1.0, 1.0), 45.0);
        let box18 = Obb2d::new(Vec2::new(2.0, 2.0), Vec2::new(0.5, 0.5), 0.0);
        let box19 = Obb2d::new(Vec2::new(2.5, 2.5), Vec2::new(0.5, 0.5), 0.0);
        let box20 = Obb2d::new(Vec2::new(-2.0, 4.0), Vec2::new(1.0, 2.0), 30.0);
        let box21 = Obb2d::new(Vec2::new(-2.5, 4.5), Vec2::new(1.0, 2.0), 30.0);
        let box22 = Obb2d::new(Vec2::new(8.0, 8.0), Vec2::new(2.0, 2.0), 60.0);
        let box23 = Obb2d::new(Vec2::new(10.0, 10.0), Vec2::new(1.0, 1.0), 60.0);
        let box24 = Obb2d::new(Vec2::new(-8.0, -8.0), Vec2::new(3.0, 1.5), 15.0);
        let box25 = Obb2d::new(Vec2::new(-9.0, -9.0), Vec2::new(1.0, 1.0), 0.0);

        assert_eq!(box1.get_intersection_result(&box5).collision, true);
        assert_eq!(box1.get_intersection_result(&box13).collision, true);
        assert_eq!(box5.get_intersection_result(&box13).collision, true);
        assert_eq!(box7.get_intersection_result(&box8).collision, true);
        assert_eq!(box9.get_intersection_result(&box10).collision, true);
        assert_eq!(box11.get_intersection_result(&box12).collision, true);
        assert_eq!(box14.get_intersection_result(&box15).collision, true);
        assert_eq!(box16.get_intersection_result(&box17).collision, true);
        assert_eq!(box18.get_intersection_result(&box19).collision, true);
        assert_eq!(box20.get_intersection_result(&box21).collision, true);
        assert_eq!(box24.get_intersection_result(&box25).collision, true);
        assert_eq!(box2.get_intersection_result(&box5).collision, true);
        assert_eq!(box5.get_intersection_result(&box10).collision, true);
        assert_eq!(box9.get_intersection_result(&box13).collision, true);
        assert_eq!(box12.get_intersection_result(&box11).collision, true);
        assert_eq!(box17.get_intersection_result(&box7).collision, true);
        assert_eq!(box1.get_intersection_result(&box2).collision, true);
        assert_eq!(box18.get_intersection_result(&box2).collision, true);
        assert_eq!(box4.get_intersection_result(&box10).collision, true);
        assert_eq!(box7.get_intersection_result(&box16).collision, true);
        
        assert_eq!(box2.get_intersection_result(&box4).collision, false);        
        assert_eq!(box6.get_intersection_result(&box21).collision, false);
        assert_eq!(box14.get_intersection_result(&box10).collision, false);
        assert_eq!(box3.get_intersection_result(&box2).collision, false);
        assert_eq!(box6.get_intersection_result(&box20).collision, false);
        assert_eq!(box19.get_intersection_result(&box5).collision, false);
        assert_eq!(box1.get_intersection_result(&box22).collision, false);
        assert_eq!(box2.get_intersection_result(&box23).collision, false);
        assert_eq!(box3.get_intersection_result(&box24).collision, false);
        assert_eq!(box4.get_intersection_result(&box6).collision, false);
        assert_eq!(box6.get_intersection_result(&box16).collision, false);
        assert_eq!(box7.get_intersection_result(&box20).collision, false);
        assert_eq!(box8.get_intersection_result(&box25).collision, false);
        assert_eq!(box9.get_intersection_result(&box23).collision, false);
        assert_eq!(box10.get_intersection_result(&box18).collision, false);
        assert_eq!(box11.get_intersection_result(&box20).collision, false);
        assert_eq!(box12.get_intersection_result(&box6).collision, false);
        assert_eq!(box13.get_intersection_result(&box21).collision, false);
        assert_eq!(box14.get_intersection_result(&box5).collision, false);
        assert_eq!(box15.get_intersection_result(&box2).collision, false);
        assert_eq!(box16.get_intersection_result(&box21).collision, false);
        assert_eq!(box17.get_intersection_result(&box6).collision, false);
        assert_eq!(box18.get_intersection_result(&box24).collision, false);
        assert_eq!(box19.get_intersection_result(&box4).collision, false);
        assert_eq!(box20.get_intersection_result(&box1).collision, false);
        assert_eq!(box21.get_intersection_result(&box3).collision, false);
        assert_eq!(box22.get_intersection_result(&box9).collision, false);
        assert_eq!(box23.get_intersection_result(&box5).collision, false);
        assert_eq!(box24.get_intersection_result(&box8).collision, false);
        assert_eq!(box25.get_intersection_result(&box11).collision, false);
    }
}