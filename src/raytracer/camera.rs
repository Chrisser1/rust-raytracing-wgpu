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

    // Moves the camera forwards or backwards
    pub fn move_forwards(&mut self, distance: f32) {
        self.position.0 += self.forwards.0 * distance;
        self.position.1 += self.forwards.1 * distance;
        self.position.2 += self.forwards.2 * distance;
    }

    // Moves the camera right or left
    pub fn move_vertical(&mut self, distance: f32) {
        self.position.0 += self.right.0 * distance;
        self.position.1 += self.right.1 * distance;
        self.position.2 += self.right.2 * distance;
    }

    // Moves the camera up or down
    pub fn move_horizontal(&mut self, distance: f32) {
        self.position.0 += self.up.0 * distance;
        self.position.1 += self.up.1 * distance;
        self.position.2 += self.up.2 * distance;
    }



    // Rotates the camera left or right
    pub fn rotate_yaw(&mut self, angle: f32) {
        self.theta += angle;
        self.recalculate_vectors();
    }

    // Rotates the camera up or down
    pub fn rotate_pitch(&mut self, angle: f32) {
        self.phi = (self.phi + angle).clamp(-89.0, 89.0); // Clamp to prevent flipping
        self.recalculate_vectors();
    }
}
