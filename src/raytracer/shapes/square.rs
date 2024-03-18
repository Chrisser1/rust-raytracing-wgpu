use winit::dpi::PhysicalSize;
use super::{Triangle, Vec3};

#[derive(Debug, Clone)]
pub struct Square {
    pub center: Vec3,
    pub size: PhysicalSize<f32>,
    pub color: Vec3,
    pub orientation: f32, // Orientation in radians around the Y-axis
    pub triangles: Vec<Triangle>,
}

impl Square {
    pub fn new(center: Vec3, height: f32, width: f32, color: Vec3, orientation: f32) -> Self {
        let mut square = Self { 
            center, 
            size: PhysicalSize::new(width, height), 
            color,
            orientation, // Set orientation
            triangles: Vec::new(),
        };

        square.calculate_triangles();

        square
    }

    fn calculate_triangles(&mut self) {
        let width = self.size.width;
        let height = self.size.height;

        // Calculate half dimensions for convenience
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        // Define unrotated corners of the square
        let mut corners = [
            Vec3(-half_width, 0.0, half_height), // Top left
            Vec3(half_width, 0.0, half_height),  // Top right
            Vec3(-half_width, 0.0, -half_height), // Bottom left
            Vec3(half_width, 0.0, -half_height), // Bottom right
        ];

        // Rotate corners around the Y-axis based on the square's orientation
        for corner in &mut corners {
            let rotated_x = corner.0 * self.orientation.cos() - corner.2 * self.orientation.sin();
            let rotated_z = corner.0 * self.orientation.sin() + corner.2 * self.orientation.cos();
            *corner = Vec3(rotated_x, corner.1, rotated_z) + self.center;
        }

        // Create two triangles from these corners and store them
        let triangle1 = Triangle::build_from_corners([corners[0], corners[2], corners[1]], self.color);
        let triangle2 = Triangle::build_from_corners([corners[2], corners[3], corners[1]], self.color);

        self.triangles.push(triangle1);
        self.triangles.push(triangle2);
    }
}
