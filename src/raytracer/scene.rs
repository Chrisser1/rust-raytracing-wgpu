use rand::Rng;

use super::{sphere, Camera, Sphere}; // Import the Rng trait to use random number generation methods

pub struct SceneData {
    pub camera_pos: (f32, f32, f32),
    pub camera_forwards: (f32, f32, f32),
    pub camera_right: (f32, f32, f32),
    pub camera_up: (f32, f32, f32),
    pub sphere_count: f32,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    camera: Camera,
}

impl Scene {
    pub fn new(sphere_count: usize) -> Self {
        let mut spheres = Vec::with_capacity(sphere_count); // Preallocate space for 32 spheres
        let mut rng = rand::thread_rng(); // Get a random number generator

        for _ in 0..sphere_count {
            let center = (
                -50.0 + 100.0 * rng.gen::<f32>(),
                -50.0 + 100.0 * rng.gen::<f32>(),
                -50.0 + 100.0 * rng.gen::<f32>(),
            );
            
            let color = (
                0.3 + 0.7 * rng.gen::<f32>(),
                0.3 + 0.7 * rng.gen::<f32>(),
                0.3 + 0.7 * rng.gen::<f32>(),
            );

            let radius = 0.1 + 1.9 * rng.gen::<f32>();

            spheres.push(Sphere::new(center, color, radius));
        }

        let camera = Camera::new((-20.0, 0.0, 0.0));

        Self { spheres, camera }
    }

    pub fn to_scene_data(&self) -> SceneData {
        SceneData {
            camera_pos: self.camera.position,
            camera_forwards: self.camera.forwards,
            camera_right: self.camera.right,
            camera_up: self.camera.up,
            sphere_count: self.spheres.len() as f32,
        }
    }

    pub fn flatten_sphere_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for sphere in &self.spheres {
            // Flatten each sphere's data into f32 values
            let sphere_attributes: [f32; 8] = [
                sphere.center.0, sphere.center.1, sphere.center.2, 0.0, // Position + Padding
                sphere.color.0, sphere.color.1, sphere.color.2, sphere.radius, // Color + Radius
            ];

            // Convert the f32 values to bytes and extend the data vector
            data.extend_from_slice(bytemuck::cast_slice(&sphere_attributes));
        }

        data
    }
}