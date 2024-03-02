#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub min_corner: (f32, f32, f32),
    pub left_child: i32,
    pub max_corner: (f32, f32, f32),
    pub sphere_count: usize,
}

impl Node {
    pub fn default() -> Self {
        Node {
            // Use very large/small numbers to indicate an "empty" bounding volume
            min_corner: (f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max_corner: (f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            left_child: -1, // Using -1 to indicate "no child"
            sphere_count: 0,
        }
    }
}
