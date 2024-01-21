use std::{
    fs::File,
    io::Write,
    ops::{Add, Mul},
};

use crate::vec3::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl From<Color> for Vec3 {
    fn from(val: Color) -> Self {
        Self::new(val.r(), val.g(), val.b())
    }
}

impl<T> Add<T> for Color
where
    T: Into<Self> + From<Self>,
{
    type Output = T;

    fn add(self, vec: T) -> Self::Output {
        let color: Self = Into::into(vec);
        Self {
            r: self.r + color.r,
            g: self.g + color.g,
            b: self.b + color.b,
        }
        .into()
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        }
    }
}

impl Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, scalar: Color) -> Self::Output {
        scalar * self
    }
}

impl Color {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub const fn r(&self) -> f64 {
        self.r
    }

    pub const fn g(&self) -> f64 {
        self.g
    }

    pub const fn b(&self) -> f64 {
        self.b
    }

    pub fn write(&self, file: &mut File) -> std::io::Result<()> {
        let colors = *self * 255.0;

        #[cfg(debug_assertions)]
        assert!(colors.r() <= 255.0);
        #[cfg(debug_assertions)]
        assert!(colors.r() >= 0.0);
        #[cfg(debug_assertions)]
        assert!(colors.g() <= 255.0);
        #[cfg(debug_assertions)]
        assert!(colors.g() >= 0.0);
        #[cfg(debug_assertions)]
        assert!(colors.b() <= 255.0);
        #[cfg(debug_assertions)]
        assert!(colors.b() >= 0.0);

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        writeln!(
            file,
            "{} {} {}",
            colors.r() as u64,
            colors.g() as u64,
            colors.b() as u64
        )?;
        Ok(())
    }
}
