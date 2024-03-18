use std::fs;
use super::{Vec3, Vec2, Triangle};

// Struct to represent an OBJ mesh
pub struct ObjMesh {
    // Vertices, texture coordinates, and normals
    v: Vec<Vec3>,
    vt: Vec<Vec2>,
    vn: Vec<Vec3>,

    pub triangles: Vec<Triangle>,
    color: Vec3,
}

impl ObjMesh {
    pub fn new(color: Vec3, path: &str) -> Self {


        let contents = fs::read_to_string(path)
            .expect("Should have been able to read the file");

        let mut mesh = ObjMesh {
            v: Vec::new(),
            vt: Vec::new(),
            vn: Vec::new(),
            triangles: Vec::new(),
            color,
        };

        mesh.process_file_contents(&contents);

        mesh
    }


    fn process_file_contents(&mut self, contents: &str) {
        let lines = contents.lines();

        for line in lines {
            if line.starts_with("v ") {
                self.read_vertex_data(line);
            } else if line.starts_with("vt") {
                self.read_texcoord_data(line);
            } else if line.starts_with("vn") {
                self.read_normal_data(line);
            } else if line.starts_with("f") {
                self.read_face_data(line);
            }
        }
    }

    fn read_vertex_data(&mut self, line: &str) {
        let components: Vec<&str> = line.split_whitespace().collect();
        // ["v", "x", "y", "z"]
        let new_vertex = Vec3(
            components[1].parse().unwrap(),
            components[2].parse().unwrap(),
            components[3].parse().unwrap(),
        );

        self.v.push(new_vertex);
    }

    fn read_texcoord_data(&mut self, line: &str) {
        let components: Vec<&str> = line.split_whitespace().collect();
        // ["vt", "u", "v"]
        let new_texcoord = Vec2(
            components[1].parse().unwrap(),
            components[2].parse().unwrap(),
        );

        self.vt.push(new_texcoord);
    }

    fn read_normal_data(&mut self, line: &str) {
        let components: Vec<&str> = line.split_whitespace().collect();
        // ["vn", "nx", "ny", "nz"]
        let new_normal = Vec3(
            components[1].parse().unwrap(),
            components[2].parse().unwrap(),
            components[3].parse().unwrap(),
        );

        self.vn.push(new_normal);
    }

    pub fn read_face_data(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Skip the "f" prefix and then process the vertices
        let vertex_descriptions = &parts[1..];
    
        // For each face, convert it into triangles
        // Assuming the face is a quad or a polygon that needs to be triangulated as a fan
        let first_vertex_description = vertex_descriptions[0];
        for i in 1..vertex_descriptions.len() - 1 {
            let mut tri = Triangle::new(); // Assuming Triangle::default() or some initializer exists
            tri.corners[0] = self.read_corner(first_vertex_description);
            tri.corners[1] = self.read_corner(vertex_descriptions[i]);
            tri.corners[2] = self.read_corner(vertex_descriptions[i + 1]);
            tri.color = self.color;
            tri.make_centroid();
            self.triangles.push(tri);
        }
    }

    fn read_corner(&mut self, vertex_description: &str) -> Vec3 {
        let v_vt_vn: Vec<&str> = vertex_description.split('/').collect();
        let v = self.v[v_vt_vn[0].parse::<usize>().unwrap() - 1];
        // let vt = self.vt[v_vt_vn[1].parse::<usize>().unwrap() - 1];
        // let vn =self.vn[v_vt_vn[2].parse::<usize>().unwrap() - 1];

        return v;
    }
}