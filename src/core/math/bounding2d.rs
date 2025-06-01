use glam::Vec2;

pub struct Aabb2d {
    min: Vec2,
    max: Vec2,
}

impl Aabb2d {
    pub fn new(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn intersects(&self, other_aabb2d: &Aabb2d) -> bool {
        let does_x_overlap = self.min.x <= other_aabb2d.max.x && self.max.x >= other_aabb2d.min.x;
        let does_y_overlap = self.min.y <= other_aabb2d.max.y && self.max.y >= other_aabb2d.min.y;
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

    use super::{Aabb2d, BoundingCircle};


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