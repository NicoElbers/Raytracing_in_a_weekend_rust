use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::space::point3::Point3;
use crate::{raytracing::color::Color, util::random::XorShift};

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

impl Mul<usize> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: usize) -> Self::Output {
        #[allow(clippy::cast_precision_loss)]
        let scalar = scalar as f64;
        self * scalar
    }
}

impl Mul<Vec3> for usize {
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
        write!(f, "{:.2} {:.2} {:.2}", self.x, self.y, self.z)
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

    #[must_use]
    pub fn is_unit(&self, delta: f64) -> bool {
        let unit_vec = self.unit();

        let x = (-delta..=delta).contains(&(self.x - unit_vec.x));
        let y = (-delta..=delta).contains(&(self.y - unit_vec.y));
        let z = (-delta..=delta).contains(&(self.z - unit_vec.z));

        x && y && z
    }

    #[must_use]
    pub fn random(rand: &mut XorShift) -> Self {
        Self {
            x: rand.next_01(),
            y: rand.next_01(),
            z: rand.next_01(),
        }
    }

    #[must_use]
    pub fn random_bounded(rand: &mut XorShift, min: f64, max: f64) -> Self {
        let diff = max - min;

        Self {
            x: min + rand.next_01() * diff,
            y: min + rand.next_01() * diff,
            z: min + rand.next_01() * diff,
        }
    }

    #[must_use]
    pub fn random_in_unit_sphere(rand: &mut XorShift) -> Self {
        loop {
            let point = Self::random_bounded(rand, -1., 1.);

            if point.len_squared() <= 1. {
                return point;
            }
        }
    }

    #[must_use]
    pub fn random_unit_vec(rand: &mut XorShift) -> Self {
        Self::random_in_unit_sphere(rand).unit()
    }

    pub fn random_vec_on_hemishpere(rand: &mut XorShift, normal: &Self) -> Self {
        let unit_vec = Self::random_unit_vec(rand);

        debug_assert!(unit_vec.is_unit(0.01));

        if Self::dot(unit_vec, *normal) > 0. {
            unit_vec
        } else {
            -unit_vec
        }
    }

    pub fn near_zero(&self) -> bool {
        let delta = 1e-8;

        (self.x < delta) && (self.y < delta) && (self.z < delta)
    }

    pub fn reflect(self, n: &Self) -> Self {
        let v = self;
        let b = Self::dot(self, *n) * *n;

        v - 2. * b
    }

    pub fn refract(self, n: &Self, refraction_ratio: f64) -> Self {
        let n = *n;

        let cos_theta = f64::min(Self::dot(-self, n), 1.);

        let out_perpendicular = refraction_ratio * (self + (cos_theta * n));
        let out_parallel = -f64::sqrt(f64::abs(1. - out_perpendicular.len_squared())) * n;

        out_perpendicular + out_parallel
    }

    pub fn random_vec_in_unit_disk(rand: &mut XorShift) -> Self {
        loop {
            let vec = Self::new(rand.next_bound(-1., 1.), rand.next_bound(-1., 1.), 0.);
            if vec.len_squared() < 1. {
                return vec;
            }
        }
    }
}
