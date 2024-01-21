use std::{
    f64::EPSILON,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::{color::Color, point3::Point3};

#[derive(Default, Debug, Clone, Copy)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self::new(value.x(), value.y(), value.z())
    }
}

impl From<Vec3> for Point3 {
    fn from(value: Vec3) -> Self {
        Self::new(value.x(), value.y(), value.z())
    }
}

impl<T> Add<T> for Vec3
where
    T: Into<Self> + From<Self>,
{
    type Output = Self;

    fn add(self, vec: T) -> Self::Output {
        let vec: Self = Into::into(vec);
        Self {
            x: self.x + vec.x,
            y: self.y + vec.y,
            z: self.z + vec.z,
        }
    }
}

impl<T> Sub<T> for Vec3
where
    T: Into<Self>,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        let vec: Self = Into::into(rhs);
        Self {
            x: self.x - vec.x,
            y: self.y - vec.y,
            z: self.z - vec.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

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
    type Output = Self;

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
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub const fn z(&self) -> f64 {
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
    pub fn dot<T>(lhs: T, rhs: T) -> f64
    where
        T: Into<Self>,
    {
        let rhs: Self = Into::into(rhs);
        let lhs: Self = Into::into(lhs);
        lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
    }

    #[must_use]
    pub fn cross<T>(&self, rhs: T) -> Self
    where
        T: Into<Self>,
    {
        let rhs: Self = Into::into(rhs);
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[must_use]
    pub fn unit(self) -> Self {
        self / self.len()
    }

    pub fn is_unit(self) -> bool {
        let unit_vec = self.unit();

        let x = (-EPSILON..=EPSILON).contains(&(self.x - unit_vec.x));
        let y = (-EPSILON..=EPSILON).contains(&(self.y - unit_vec.y));
        let z = (-EPSILON..=EPSILON).contains(&(self.z - unit_vec.z));

        x && y && z
    }
}
