use std::collections::HashSet;

use rand::Rng;
use winit::keyboard::KeyCode;

use super::{Camera, Node, ObjMesh, Sphere, Triangle, Vec3}; // Import the Rng trait to use random number generation methods

pub enum Object {
    Sphere(Sphere),
    Triangle(Triangle),
}

pub struct Scene {
    pub objects: Vec<Object>,
    pub camera: Camera,
    pub nodes: Vec<Node>,
    pub nodes_used: usize,
    pub object_indices: Vec<usize>,
    pub max_bounces: usize,
    pub keys_pressed: HashSet<KeyCode>,
    pub object_meshes: Vec<ObjMesh>,
}

impl Scene {
    pub fn new(object_count: usize, max_bounces: usize) -> Self {
        let mut scene = Self {
            objects: Vec::with_capacity(object_count),
            camera: Camera::new(Vec3(-5.0, 0.0, 0.0)),
            nodes: Vec::with_capacity(2 * object_count - 1),
            nodes_used: 0,
            object_indices: Vec::with_capacity(object_count),
            max_bounces: max_bounces,
            keys_pressed: HashSet::new(),
            object_meshes: Vec::new(),
        };

        let golden_angle = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt()); // Golden angle in radians
        let radius = 50.0; // Radius of the imaginary sphere on which to place the spheres

        for i in 0..object_count {
            let theta = golden_angle * i as f32; // Angle around the spiral
            let z = 1.0 - (i as f32) / (object_count as f32 - 1.0) * 2.0; // Z-coordinate varies linearly from 1 to -1
            let x = theta.cos() * (1.0 - z * z).sqrt();
            let y = theta.sin() * (1.0 - z * z).sqrt();

            let center = Vec3(radius * x, radius * y, radius * z);
            let mut rng = rand::thread_rng(); // Get a random number generator for colors

            let color = Vec3(
                0.3 + 0.7 * rng.gen::<f32>(),
                0.3 + 0.7 * rng.gen::<f32>(),
                0.3 + 0.7 * rng.gen::<f32>(),
            );


            if i % 2 == 0 {
                // Add a sphere...
                let sphere_radius = 0.1 + 1.9 * rng.gen::<f32>(); // Random sphere radius
                let sphere = Sphere::new(center, color, sphere_radius);
                scene.objects.push(Object::Sphere(sphere));
            } else {
                // Add a triangle...
                let offsets = [
                Vec3(-3.0 + 6.0 * rng.gen::<f32>(), -3.0 + 6.0 * rng.gen::<f32>(), -3.0 + 6.0 * rng.gen::<f32>()),
                Vec3(-3.0 + 6.0 * rng.gen::<f32>(), -3.0 + 6.0 * rng.gen::<f32>(), -3.0 + 6.0 * rng.gen::<f32>()),
                Vec3(-3.0 + 6.0 * rng.gen::<f32>(), -3.0 + 6.0 * rng.gen::<f32>(), -3.0 + 6.0 * rng.gen::<f32>())
                ];
                let triangle = Triangle::build_from_center_and_offsets(center, offsets, color);
                scene.objects.push(Object::Triangle(triangle));
            }
        }


        // build the scene
        scene.make_scene();

        scene
    }

    fn make_scene(&mut self) {
        self.object_meshes.push(ObjMesh::new(Vec3(1.0, 1.0, 1.0), "assets/models/statue.obj"));

        for objects in &self.object_meshes {
            for triangle in &objects.triangles {
                self.objects.push(Object::Triangle(triangle.clone()));
            }
        }

        // Initialize object indices for easy tracking
        self.object_indices = (0..self.objects.len()).collect();

        // Now, build the BVH for the scene
        self.build_bvh();
    }

    fn build_bvh(&mut self) {
        // Initialize sphere indices for easy tracking
        self.object_indices = (0..self.objects.len()).collect();
        self.nodes = vec![Node::default(); 2 * self.objects.len() - 1]; // Placeholder for actual size
        
        let root_index = 0;
        let node = &mut self.nodes[root_index];
        node.left_child = 0; // Starting index for sphere indices
        node.object_count = self.objects.len();
        self.nodes_used = 1;
        
        self.update_bounds(root_index);
        self.subdivide(root_index);
    }

    fn update_bounds(&mut self, node_index: usize) {
        let node = &mut self.nodes[node_index];

        let start_index = node.left_child as usize;
        let end_index = start_index + node.object_count as usize;

        for &i in &self.object_indices[start_index..end_index] {
            match &self.objects[i] {
                Object::Sphere(sphere) => {
                    let min = sphere.center - sphere.radius;

                    let max = sphere.center + sphere.radius;

                    node.min_corner.0 = node.min_corner.0.min(min.0);
                    node.max_corner.0 = node.max_corner.0.max(max.0);

                    node.min_corner.1 = node.min_corner.1.min(min.1);
                    node.max_corner.1 = node.max_corner.1.max(max.1);

                    node.min_corner.2 = node.min_corner.2.min(min.2);
                    node.max_corner.2 = node.max_corner.2.max(max.2);
                },
                Object::Triangle(triangle) => {
                    for corner in &triangle.corners {
                        node.min_corner.0 = node.min_corner.0.min(corner.0);
                        node.max_corner.0 = node.max_corner.0.max(corner.0);

                        node.min_corner.1 = node.min_corner.1.min(corner.1);
                        node.max_corner.1 = node.max_corner.1.max(corner.1);

                        node.min_corner.2 = node.min_corner.2.min(corner.2);
                        node.max_corner.2 = node.max_corner.2.max(corner.2);
                    }
                },
            }
            
        }
    }

    fn subdivide(&mut self, node_index: usize) {
        if self.nodes[node_index].object_count <= 2 {
            return; // Base case: node is sufficiently small
        }

        let axis = self.longest_axis(node_index);
        let (split, i) = self.median_split(node_index, axis);

        if (split == 0 || split == self.nodes[node_index].object_count as usize) {
            return;
        }

        let left_child_index = self.nodes_used;
        self.nodes_used += 1;
        let right_child_index = self.nodes_used;
        self.nodes_used += 1;

        self.nodes[left_child_index].left_child = self.nodes[node_index].left_child;
        self.nodes[left_child_index].object_count = split;

        self.nodes[right_child_index].left_child = i as i32;
        self.nodes[right_child_index].object_count = self.nodes[node_index].object_count - split;
        
        self.nodes[node_index].left_child = left_child_index as i32; // Points to its first child instead
        self.nodes[node_index].object_count = 0; // And has no direct sphere count
        
        // Recurse for each child
        self.update_bounds(left_child_index);
        self.update_bounds(right_child_index);
        self.subdivide(left_child_index);
        self.subdivide(right_child_index);
    }

    fn longest_axis(&self, node_index: usize) -> usize {
        let node = &self.nodes[node_index];
        let extent = node.max_corner - node.min_corner;
    
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
        let extent = node.max_corner - node.min_corner;

        
        let split_pos = match axis {
            0 => node.min_corner.0 + extent.0 / 2.0,
            1 => node.min_corner.1 + extent.1 / 2.0,
            _ => node.min_corner.2 + extent.2 / 2.0,
        };
    
        let start = node.left_child as usize;
        let end = start + node.object_count as usize;
        let mut i = start;
        let mut j = end - 1;
        
        while i <= j {
            if self.object_position(&self.objects[self.object_indices[i]], axis) < split_pos {
                i += 1;
            } else {
                self.object_indices.swap(i, j);
                if j == 0 { break; } // Prevent underflow
                j -= 1;
            }
        }
        
        (i - start, i) // Return the count of spheres in the left partition
    }

    fn object_position(&self, object: &Object, axis: usize) -> f32 {
        match object {
            Object::Sphere(sphere) => {
                match axis {
                    0 => sphere.center.0,
                    1 => sphere.center.1,
                    _ => sphere.center.2,
                }
            },
            Object::Triangle(triangle) => {
                // For a triangle, use the centroid or an average position of its corners for sorting
                match axis {
                    0 => triangle.centroid.0,
                    1 => triangle.centroid.1,
                    _ => triangle.centroid.2,
                }
            },
        }
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
            self.object_indices.len() as f32,
        ];

        // Convert the f32 array to bytes and return
        bytemuck::cast_slice(&scene_data_flat).to_vec()
    }

    pub fn flatten_object_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for object in &self.objects {
            match object {
                Object::Sphere(sphere) => {
                    let sphere_attributes: [f32; 17] = [
                        0.0, sphere.center.0, sphere.center.1, sphere.center.2, sphere.radius, // Center + Radius
                        sphere.color.0, sphere.color.1, sphere.color.2, // Color + Padding
                        // Padding or default values for triangle attributes
                        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                    ];
                    data.extend_from_slice(bytemuck::cast_slice(&sphere_attributes));
                },
                Object::Triangle(triangle) => {
                    let triangle_attributes: [f32; 17] = [
                        // Padding or default values for triangle attributes
                        1.0, 0.0, 0.0, 0.0, 0.0,
                        triangle.color.0, triangle.color.1, triangle.color.2, // Color + Padding
                        triangle.corners[0].0, triangle.corners[0].1, triangle.corners[0].2, // corner_a
                        triangle.corners[1].0, triangle.corners[1].1, triangle.corners[1].2, // corner_b
                        triangle.corners[2].0, triangle.corners[2].1, triangle.corners[2].2, // corner_c
                    ];
                    data.extend_from_slice(bytemuck::cast_slice(&triangle_attributes));
                },
            }
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
                node.object_count as f32, // Cast to f32 for buffer compatibility
            ];
    
            // Convert the f32 values to bytes and extend the data vector
            data.extend_from_slice(bytemuck::cast_slice(&node_attributes));
        }
    
        data
    }

    pub fn flatten_object_index_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
    
        for &index in &self.object_indices {
            // Cast each index to f32 and extend the data vector
            data.extend_from_slice(bytemuck::cast_slice(&[index as f32]));
        }
    
        data
    }

    pub fn update(&mut self) {
        let movement_speed = 0.1; // Adjust speed as necessary
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
                KeyCode::ArrowLeft => self.camera.rotate_yaw(movement_speed*4.0),
                KeyCode::ArrowRight => self.camera.rotate_yaw(-movement_speed*4.0),
                KeyCode::ArrowUp => self.camera.rotate_pitch(-movement_speed*4.0),
                KeyCode::ArrowDown => self.camera.rotate_pitch(movement_speed*4.0),
                _ => {},
            }
        }
    }
    
}