use super::Vec3;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub corners: [Vec3; 3],
    pub color: Vec3,
    pub centroid: Vec3,
}

impl Triangle {
    pub fn new() -> Self {
        let corners = [Vec3(0.0, 0.0, 0.0); 3];
        let color = Vec3(0.0, 0.0, 0.0);
        let centroid = Vec3(0.0, 0.0, 0.0);
        Self {
            corners,
            color,
            centroid
        }
    }
    pub fn build_from_center_and_offsets(center: Vec3, offsets: [Vec3; 3], color: Vec3) -> Self {
        let mut corners = [Vec3(0.0, 0.0, 0.0); 3];
        let mut centroid = Vec3(0.0, 0.0, 0.0);
        let weight = Vec3(0.33333, 0.33333, 0.33333);

        for (i, offset) in offsets.iter().enumerate() {
            corners[i] = Vec3(
                center.0 + offset.0,
                center.1 + offset.1,
                center.2 + offset.2,
            );
            centroid.0 += corners[i].0 * weight.0;
            centroid.1 += corners[i].1 * weight.1;
            centroid.2 += corners[i].2 * weight.2;
        }

        Self {
            corners,
            color,
            centroid,
        }
    }

    pub fn make_centroid(&mut self) {
        self.centroid = Vec3(
            (self.corners[0].0 + self.corners[1].0 + self.corners[2].0) / 3.0,
            (self.corners[0].1 + self.corners[1].1 + self.corners[2].1) / 3.0,
            (self.corners[0].2 + self.corners[1].2 + self.corners[2].2) / 3.0
        );
    }
}