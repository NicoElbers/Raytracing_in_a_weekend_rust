use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    ops::{Add, Div, Mul},
};

use crate::space::vec3::Vec3;

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

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Div<f64> for Color {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self {
            r: self.r / scalar,
            g: self.g / scalar,
            b: self.b / scalar,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.r, self.g, self.b)
    }
}

impl Color {
    #[must_use]
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub const fn r(&self) -> f64 {
        self.r
    }

    #[must_use]
    pub const fn g(&self) -> f64 {
        self.g
    }

    #[must_use]
    pub const fn b(&self) -> f64 {
        self.b
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn write(&self, writer: &mut BufWriter<&File>) -> std::io::Result<()> {
        debug_assert!(self.r() <= 1.0, "Invalid color: {self}");
        debug_assert!(self.r() >= 0.0, "Invalid color: {self}");
        debug_assert!(self.g() <= 1.0, "Invalid color: {self}");
        debug_assert!(self.g() >= 0.0, "Invalid color: {self}");
        debug_assert!(self.b() <= 1.0, "Invalid color: {self}");
        debug_assert!(self.b() >= 0.0, "Invalid color: {self}");

        let colors = self.gamma_correct();
        let colors = colors * 255.0;

        debug_assert!(
            colors.r() <= 255.0,
            "Invalid color after correction: {colors}"
        );
        debug_assert!(
            colors.r() >= 0.0,
            "Invalid color after correction: {colors}"
        );
        debug_assert!(
            colors.g() <= 255.0,
            "Invalid color after correction: {colors}"
        );
        debug_assert!(
            colors.g() >= 0.0,
            "Invalid color after correction: {colors}"
        );
        debug_assert!(
            colors.b() <= 255.0,
            "Invalid color after correction: {colors}"
        );
        debug_assert!(
            colors.b() >= 0.0,
            "Invalid color after correction: {colors}"
        );

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        writeln!(
            writer,
            "{} {} {}",
            colors.r() as u64,
            colors.g() as u64,
            colors.b() as u64
        )?;

        Ok(())
    }

    fn gamma_correct(self) -> Self {
        Self {
            r: self.r().powf(1. / 2.2),
            g: self.g().powf(1. / 2.2),
            b: self.b().powf(1. / 2.2),
        }
    }
}
