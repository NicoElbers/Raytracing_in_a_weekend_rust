use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Add<&Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, vec: &Vec3) -> Self::Output {
        Self {
            x: self.x + vec.x,
            y: self.y + vec.y,
            z: self.z + vec.z,
        }
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, vec: Vec3) -> Self::Output {
        Self {
            x: self.x + vec.x,
            y: self.y + vec.y,
            z: self.z + vec.z,
        }
    }
}

impl Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, vec: &Vec3) -> Self::Output {
        Self {
            x: self.x - vec.x,
            y: self.y - vec.y,
            z: self.z - vec.z,
        }
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, vec: Vec3) -> Self::Output {
        Self {
            x: self.x - vec.x,
            y: self.y - vec.y,
            z: self.z - vec.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Self::Output {
        vec * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

impl Vec3 {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub fn z(&self) -> f64 {
        self.z
    }

    #[must_use]
    pub fn len_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[must_use]
    pub fn len(&self) -> f64 {
        self.len_squared().sqrt()
    }

    #[must_use]
    pub fn dot(&self, vec: &Vec3) -> f64 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    #[must_use]
    pub fn cross(&self, rhs: &Vec3) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[must_use]
    pub fn unit(&self) -> Self {
        *self / self.len()
    }
}
