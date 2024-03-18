use super::{rotate_vector_around_axis, Vec3};

pub struct Camera {
    pub origin: Vec3,
    pub lower_left_corner: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub lens_radius: f32,
    aspect_ratio: f32,
    vfov: f32, // vertical field of view in degrees
    lookfrom: Vec3,
    lookat: Vec3,
    vup: Vec3, // up vector
}

impl Camera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, vup: Vec3, vfov: f32, aspect_ratio: f32) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = lookfrom;
        let horizontal = u * viewport_width;
        let vertical = v * viewport_height;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            lens_radius: 0.0, // Placeholder, assuming no lens distortion
            aspect_ratio,
            vfov,
            lookfrom,
            lookat,
            vup,
        }
    }

    // Correctly moves the camera forwards or backwards along the viewing direction
    pub fn move_forwards(&mut self, distance: f32) {
        let direction = (self.lookat - self.lookfrom).normalize();
        self.lookfrom += direction * distance;
        self.lookat += direction * distance;
        self.update_camera();
    }

    // Correctly moves the camera right or left
    pub fn move_vertical(&mut self, distance: f32) {
        let right = self.vup.cross(self.lookat - self.lookfrom).normalize();
        self.lookfrom += right * distance;
        self.lookat += right * distance;
        self.update_camera();
    }

    // Correctly moves the camera up or down
    pub fn move_horizontal(&mut self, distance: f32) {
        self.lookfrom += self.vup * distance;
        self.lookat += self.vup * distance;
        self.update_camera();
    }

    // Additional helper function to recalculate camera vectors after movement or rotation
    fn update_camera(&mut self) {
        let theta = self.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = self.aspect_ratio * viewport_height;

        let w = (self.lookfrom - self.lookat).normalize();
        let u = self.vup.cross(w).normalize();
        let v = w.cross(u);

        self.horizontal = u * viewport_width;
        self.vertical = v * viewport_height;
        self.lower_left_corner = self.lookfrom - self.horizontal / 2.0 - self.vertical / 2.0 - w;

        self.origin = self.lookfrom;
    }

    // Rotates the camera left or right
    pub fn rotate_yaw(&mut self, angle_deg: f32) {
        let angle_rad = angle_deg.to_radians();
        let direction = self.lookat - self.lookfrom;
        let rotated_direction = rotate_vector_around_axis(direction, self.vup, angle_rad);
        self.lookat = self.lookfrom + rotated_direction;
        self.update_camera();
    }

    // Rotates the camera up or down
    pub fn rotate_pitch(&mut self, angle_deg: f32) {
        let angle_rad = angle_deg.to_radians();
        let direction = self.lookat - self.lookfrom;
        let right = self.vup.cross(direction).normalize();
        let rotated_direction = rotate_vector_around_axis(direction, right, angle_rad);
        // Ensure the rotated direction does not flip over vertically
        let new_lookat = self.lookfrom + rotated_direction;
        if self.vup.cross(new_lookat - self.lookfrom).dot(right) > 0.0 {
            self.lookat = new_lookat;
            self.update_camera();
        }
    }
}
