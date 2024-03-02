pub struct Camera {
    pub position: (f32, f32, f32),
    pub theta: f32,
    pub phi: f32,
    pub forwards: (f32, f32, f32),
    pub right: (f32, f32, f32),
    pub up: (f32, f32, f32),
}

impl Camera {
    pub fn new(position: (f32, f32, f32)) -> Self {
        let mut camera = Self {
            position,
            theta: 0.0,
            phi: 0.0,
            forwards: (0.0, 0.0, 0.0),
            right: (0.0, 0.0, 0.0),
            up: (0.0, 0.0, 0.0),
        };
        camera.recalculate_vectors();
        camera
    }

    fn recalculate_vectors(&mut self) {
        let theta_rad = self.theta * std::f32::consts::PI / 180.0;
        let phi_rad = self.phi * std::f32::consts::PI / 180.0;

        self.forwards = (
            theta_rad.cos() * phi_rad.cos(),
            theta_rad.sin() * phi_rad.cos(),
            phi_rad.sin(),
        );

        // Simple cross product calculation for 'right' based on 'forwards' and a 'up' vector pointing up.
        self.right = self.cross(self.forwards, (0.0, 0.0, 1.0));
        self.up = self.cross(self.right, self.forwards);
    }

    // A minimal implementation of cross product for 3D vectors
    fn cross(&self, v1: (f32, f32, f32), v2: (f32, f32, f32)) -> (f32, f32, f32) {
        (
            v1.1 * v2.2 - v1.2 * v2.1,
            v1.2 * v2.0 - v1.0 * v2.2,
            v1.0 * v2.1 - v1.1 * v2.0,
        )
    }
}
