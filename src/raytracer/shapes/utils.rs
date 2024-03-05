#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl Vec3 {
    // Add two vectors
    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }

    // Subtract two vectors
    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }

    // Add a scalar to each component of the vector
    pub fn add_scalar(self, scalar: f32) -> Vec3 {
        Vec3(self.0 + scalar, self.1 + scalar, self.2 + scalar)
    }

    // Subtract a scalar from each component of the vector
    pub fn sub_scalar(self, scalar: f32) -> Vec3 {
        Vec3(self.0 - scalar, self.1 - scalar, self.2 - scalar)
    }

    // Divide each component of the vector by a scalar
    pub fn div(self, scalar: f32) -> Vec3 {
        Vec3(self.0 / scalar, self.1 / scalar, self.2 / scalar)
    }

    // Multiply vector by a scalar
    pub fn mul(self, scalar: f32) -> Vec3 {
        Vec3(self.0 * scalar, self.1 * scalar, self.2 * scalar)
    }

    // Dot product of two vectors
    pub fn dot(self, other: Vec3) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    // Cross product of two vectors
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }

    // Magnitude (length) of the vector
    pub fn magnitude(self) -> f32 {
        f32::sqrt(self.dot(self))
    }

    // Normalize the vector to unit length
    pub fn normalize(self) -> Vec3 {
        let mag = self.magnitude();
        if mag > 0.0 {
            self.mul(1.0 / mag)
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2(pub f32, pub f32);

impl Vec2 {
    // Add two vectors
    pub fn add(self, other: Vec2) -> Vec2 {
        Vec2(self.0 + other.0, self.1 + other.1)
    }

    // Subtract two vectors
    pub fn sub(self, other: Vec2) -> Vec2 {
        Vec2(self.0 - other.0, self.1 - other.1)
    }

    // Add a scalar to each component of the vector
    pub fn add_scalar(self, scalar: f32) -> Vec2 {
        Vec2(self.0 + scalar, self.1 + scalar)
    }

    // Subtract a scalar from each component of the vector
    pub fn sub_scalar(self, scalar: f32) -> Vec2 {
        Vec2(self.0 - scalar, self.1 - scalar)
    }

    // Divide each component of the vector by a scalar
    pub fn div(self, scalar: f32) -> Vec2 {
        Vec2(self.0 / scalar, self.1 / scalar)
    }

    // Multiply vector by a scalar
    pub fn mul(self, scalar: f32) -> Vec2 {
        Vec2(self.0 * scalar, self.1 * scalar)
    }

    // Dot product of two vectors
    pub fn dot(self, other: Vec2) -> f32 {
        self.0 * other.0 + self.1 * other.1
    }

    // Magnitude (length) of the vector
    pub fn magnitude(self) -> f32 {
        f32::sqrt(self.dot(self))
    }

    // Normalize the vector to unit length
    pub fn normalize(self) -> Vec2 {
        let mag = self.magnitude();
        if mag > 0.0 {
            self.mul(1.0 / mag)
        } else {
            self
        }
    }
}

// Implementing std::ops traits for syntactic sugar
use std::ops::{Add, Sub, Mul, Div};

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Self) -> Self::Output {
        self.add(other)
    }
}

impl Add<f32> for Vec3 {
    type Output = Vec3;

    fn add(self, scalar: f32) -> Self::Output {
        self.add_scalar(scalar)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Self) -> Self::Output {
        self.sub(other)
    }
}

impl Sub<f32> for Vec3 {
    type Output = Vec3;

    fn sub(self, scalar: f32) -> Self::Output {
        self.sub_scalar(scalar)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f32) -> Self::Output {
        self.mul(scalar)
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, scalar: f32) -> Self::Output {
        self.div(scalar)
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Self) -> Self::Output {
        self.add(other)
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;

    fn add(self, scalar: f32) -> Self::Output {
        self.add_scalar(scalar)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Self) -> Self::Output {
        self.sub(other)
    }
}

impl Sub<f32> for Vec2 {
    type Output = Vec2;

    fn sub(self, scalar: f32) -> Self::Output {
        self.sub_scalar(scalar)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Self::Output {
        self.mul(scalar)
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, scalar: f32) -> Self::Output {
        self.div(scalar)
    }
}