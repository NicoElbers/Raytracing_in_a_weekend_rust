use std::{ops::{Add, Mul, Sub, Div}, fmt::Display};

use crate::vec3::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl From<Point3> for Vec3 {
    fn from(value: Point3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl<T> Add<T> for Point3
where
    T: Into<Self> + From<Self>,
{
    type Output = Self;

    fn add(self, point: T) -> Self::Output {
        let point: Self = Into::into(point);
        Self {
            x: self.x + point.x,
            y: self.y + point.y,
            z: self.z + point.z,
        }
    }
}

impl<T> Sub<T> for Point3
where
    T: Into<Vec3>,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        let rhs: Vec3 = rhs.into();

        Self {
            x: self.x - rhs.x(),
            y: self.y - rhs.y(),
            z: self.z - rhs.z(),
        }
    }
}

impl Mul<f64> for Point3 {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Point3> for f64 {
    type Output = Point3;

    fn mul(self, scalar: Point3) -> Self::Output {
        scalar * self
    }
}

impl Div<f64> for Point3 {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl Display for Point3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

