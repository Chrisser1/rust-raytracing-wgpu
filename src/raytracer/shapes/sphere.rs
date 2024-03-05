use super::Vec3;

// Sphere struct that implements the Shape trait
pub struct Sphere {
    pub center: Vec3,
    pub color: Vec3,
    pub radius: f32,
}

impl Sphere {
    // Sphere constructor
    pub fn new(center: Vec3, color: Vec3, radius: f32) -> Self {
        Self { center, color, radius }
    }
}