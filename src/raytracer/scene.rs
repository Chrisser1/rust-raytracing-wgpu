use std::collections::HashSet;

use rand::Rng;
use winit::keyboard::KeyCode;

use super::{Camera, Node, Sphere}; // Import the Rng trait to use random number generation methods

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub camera: Camera,
    pub nodes: Vec<Node>,
    pub nodes_used: usize,
    pub sphere_indices: Vec<usize>,
    pub max_bounces: usize,
    pub keys_pressed: HashSet<KeyCode>,
}

impl Scene {
    pub fn new(sphere_count: usize, max_bounces: usize) -> Self {
        let mut scene = Self {
            spheres: Vec::with_capacity(sphere_count),
            camera: Camera::new((-20.0, 0.0, 0.0)),
            nodes: Vec::with_capacity(2 * sphere_count - 1),
            nodes_used: 0,
            sphere_indices: Vec::with_capacity(sphere_count),
            max_bounces: max_bounces,
            keys_pressed: HashSet::new(),
        };

        let golden_angle = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt()); // Golden angle in radians
        let radius = 50.0; // Radius of the imaginary sphere on which to place the spheres

        for i in 0..sphere_count {
            let theta = golden_angle * i as f32; // Angle around the spiral
            let z = 1.0 - (i as f32) / (sphere_count as f32 - 1.0) * 2.0; // Z-coordinate varies linearly from 1 to -1
            let x = theta.cos() * (1.0 - z * z).sqrt();
            let y = theta.sin() * (1.0 - z * z).sqrt();

            let center = (radius * x, radius * y, radius * z);

            let mut rng = rand::thread_rng(); // Get a random number generator for colors
            let color = (
                0.3 + 0.7 * rng.gen::<f32>(),
                0.3 + 0.7 * rng.gen::<f32>(),
                0.3 + 0.7 * rng.gen::<f32>(),
            );

            let sphere_radius = 0.1 + 1.9 * rng.gen::<f32>(); // Random sphere radius

            scene.spheres.push(Sphere::new(center, color, sphere_radius));
        }

        // Initialize sphere indices for easy tracking
        scene.sphere_indices = (0..sphere_count).collect();

        // Now, build the BVH for the scene
        scene.build_bvh();

        scene
    }

    fn build_bvh(&mut self) {
        // Initialize sphere indices for easy tracking
        self.sphere_indices = (0..self.spheres.len()).collect();
        self.nodes = vec![Node::default(); 2 * self.spheres.len() - 1]; // Placeholder for actual size
        
        let root_index = 0;
        let node = &mut self.nodes[root_index];
        node.left_child = 0; // Starting index for sphere indices
        node.sphere_count = self.spheres.len();
        self.nodes_used = 1;
        
        self.update_bounds(root_index);
        self.subdivide(root_index);
    }

    fn update_bounds(&mut self, node_index: usize) {
        let node = &mut self.nodes[node_index];

        let start_index = node.left_child as usize;
        let end_index = start_index + node.sphere_count as usize;

        for &i in &self.sphere_indices[start_index..end_index] {
            let sphere = &self.spheres[i];
            let min = [
                sphere.center.0 - sphere.radius,
                sphere.center.1 - sphere.radius,
                sphere.center.2 - sphere.radius,
            ];
            let max = [
                sphere.center.0 + sphere.radius,
                sphere.center.1 + sphere.radius,
                sphere.center.2 + sphere.radius,
            ];

            node.min_corner.0 = node.min_corner.0.min(min[0]);
            node.max_corner.0 = node.max_corner.0.max(max[0]);

            node.min_corner.1 = node.min_corner.1.min(min[1]);
            node.max_corner.1 = node.max_corner.1.max(max[1]);

            node.min_corner.2 = node.min_corner.2.min(min[2]);
            node.max_corner.2 = node.max_corner.2.max(max[2]);
        }
    }

    fn subdivide(&mut self, node_index: usize) {
        if self.nodes[node_index].sphere_count <= 2 {
            return; // Base case: node is sufficiently small
        }

        let axis = self.longest_axis(node_index);
        let (split, i) = self.median_split(node_index, axis);

        if (split == 0 || split == self.nodes[node_index].sphere_count as usize) {
            return;
        }

        let left_child_index = self.nodes_used;
        self.nodes_used += 1;
        let right_child_index = self.nodes_used;
        self.nodes_used += 1;

        self.nodes[left_child_index].left_child = self.nodes[node_index].left_child;
        self.nodes[left_child_index].sphere_count = split;

        self.nodes[right_child_index].left_child = i as i32;
        self.nodes[right_child_index].sphere_count = self.nodes[node_index].sphere_count - split;
        
        self.nodes[node_index].left_child = left_child_index as i32; // Points to its first child instead
        self.nodes[node_index].sphere_count = 0; // And has no direct sphere count
        
        // Recurse for each child
        self.update_bounds(left_child_index);
        self.update_bounds(right_child_index);
        self.subdivide(left_child_index);
        self.subdivide(right_child_index);
    }

    fn longest_axis(&self, node_index: usize) -> usize {
        let node = &self.nodes[node_index];
        let extent = (
            node.max_corner.0 - node.min_corner.0,
            node.max_corner.1 - node.min_corner.1,
            node.max_corner.2 - node.min_corner.2,
        );
    
        if extent.0 > extent.1 && extent.0 > extent.2 {
            0
        } else if extent.1 > extent.2 {
            1
        } else {
            2
        }
    }

    fn median_split(&mut self, node_index: usize, axis: usize) -> (usize, usize) {
        let node = &self.nodes[node_index];
        let extent = (
            node.max_corner.0 - node.min_corner.0,
            node.max_corner.1 - node.min_corner.1,
            node.max_corner.2 - node.min_corner.2,
        );
        
        let split_pos = match axis {
            0 => node.min_corner.0 + extent.0 / 2.0,
            1 => node.min_corner.1 + extent.1 / 2.0,
            _ => node.min_corner.2 + extent.2 / 2.0,
        };
    
        let start = node.left_child as usize;
        let end = start + node.sphere_count as usize;
        let mut i = start;
        let mut j = end - 1;
        
        while i <= j {
            if self.sphere_position(&self.spheres[self.sphere_indices[i]], axis) < split_pos {
                i += 1;
            } else {
                self.sphere_indices.swap(i, j);
                if j == 0 { break; } // Prevent underflow
                j -= 1;
            }
        }
        
        (i - start, i) // Return the count of spheres in the left partition
    }

    fn sphere_position(&self, sphere: &Sphere, axis: usize) -> f32 {
        let pos = match axis {
            0 => sphere.center.0,
            1 => sphere.center.1,
            _ => sphere.center.2,
        };
        pos
    }

    pub fn flatten_scene_data(&self) -> Vec<u8> {
        let scene_data_flat: [f32; 16] = [
            self.camera.position.0,
            self.camera.position.1,
            self.camera.position.2,
            0.0, // Padding for alignment
            self.camera.forwards.0,
            self.camera.forwards.1,
            self.camera.forwards.2,
            0.0, // Padding for alignment
            self.camera.right.0,
            self.camera.right.1,
            self.camera.right.2,
            self.max_bounces as f32,
            self.camera.up.0,
            self.camera.up.1,
            self.camera.up.2,
            self.spheres.len() as f32,
        ];

        // Convert the f32 array to bytes and return
        bytemuck::cast_slice(&scene_data_flat).to_vec()
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

    pub fn flatten_node_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
    
        for i in 0..self.nodes_used {
            let node = &self.nodes[i];
            // Flatten each node's data into f32 values
            let node_attributes: [f32; 8] = [
                node.min_corner.0, node.min_corner.1, node.min_corner.2,
                node.left_child as f32, // Cast to f32 for buffer compatibility
                node.max_corner.0, node.max_corner.1, node.max_corner.2,
                node.sphere_count as f32, // Cast to f32 for buffer compatibility
            ];
    
            // Convert the f32 values to bytes and extend the data vector
            data.extend_from_slice(bytemuck::cast_slice(&node_attributes));
        }
    
        data
    }

    pub fn flatten_sphere_index_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
    
        for &index in &self.sphere_indices {
            // Cast each index to f32 and extend the data vector
            data.extend_from_slice(bytemuck::cast_slice(&[index as f32]));
        }
    
        data
    }

    pub fn update(&mut self) {
        let movement_speed = 1.0; // Adjust speed as necessary
        for key in self.keys_pressed.iter() {
            match key {
                KeyCode::KeyW => self.camera.move_forwards(movement_speed),
                KeyCode::KeyS => self.camera.move_forwards(-movement_speed),
                KeyCode::KeyA => self.camera.move_vertical(-movement_speed),
                KeyCode::KeyD => self.camera.move_vertical(movement_speed),
                KeyCode::KeyQ => self.camera.move_horizontal(-movement_speed),
                KeyCode::KeyE => self.camera.move_horizontal(movement_speed),
                KeyCode::Space => self.camera.move_horizontal(-movement_speed),
                KeyCode::ShiftLeft => self.camera.move_horizontal(movement_speed),
                KeyCode::ArrowLeft => self.camera.rotate_yaw(movement_speed/2.0),
                KeyCode::ArrowRight => self.camera.rotate_yaw(-movement_speed/2.0),
                KeyCode::ArrowUp => self.camera.rotate_pitch(-movement_speed/2.0),
                KeyCode::ArrowDown => self.camera.rotate_pitch(movement_speed/2.0),
                _ => {},
            }
        }
    }
    
}