use crate::raytracer::Shape;

// Sphere struct that implements the Shape trait
pub struct Sphere {
    pub center: (f32, f32, f32),
    pub color: (f32, f32, f32),
    pub radius: f32,
}

// Implement the Shape trait for Sphere
impl Shape for Sphere {
    fn describe(&self) -> String {
        format!("Sphere with center {:?}, radius {}, and color {:?}", self.center, self.radius, self.color)
    }
}

impl Sphere {
    // Sphere constructor
    pub fn new(center: (f32, f32, f32), color: (f32, f32, f32), radius: f32) -> Self {
        Self { center, color, radius }
    }
}