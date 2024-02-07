use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    ops::{Add, Div, Mul},
};

use crate::{space::vec3::Vec3, util::random::XorShift};

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

impl Mul<Self> for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
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

        // println!("Writing color {colors:?}");

        Ok(())
    }

    pub fn write_colors(colors: &[Self], writer: &mut BufWriter<&File>) -> std::io::Result<()> {
        let str_len = colors.len() * 6;


        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let mut bytes = colors
            .iter()
            .map(|color| color.gamma_correct())
            .flat_map(|color| vec![color.r(), color.g(), color.b()])
            .map(|val| val * 255.)
            .map(|color_value| color_value as u64)
            .map(|val| val.to_string())
            .fold(String::with_capacity(str_len), |acc, val| acc + &val + " ");

        if bytes.pop().is_some() {
            bytes.push('\n');
        }

        let bytes = bytes.as_bytes();

        writer.write_all(bytes)?;
        writer.flush()?;

        Ok(())
    }

    pub fn wire_full_file(image: &mut Vec<Vec<Self>>, writer: &mut BufWriter<&File>) -> std::io::Result<()> {
        let height = image.len();
        let width = image[0].len();
        
        // Get image string
        // Prelude string
        let prelude_string = format!("P3\n{width} {height}\n255\n");

        // 3 bytes for the digits, ond byte for the space/ newline
        let color_bytes = height * width * 4;

        let mut image_string = String::with_capacity(color_bytes + prelude_string.as_bytes().len());
        image_string.push_str(&prelude_string);

        for line in image{
            // Turn line into string of color values
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let mut line_string = line.iter()
                .map(|color| color.gamma_correct())
                .flat_map(|color| vec![color.r(), color.g(), color.b()])
                .map(|val| val * 255.)
                .map(|val| val as u64)
                .map(|val| val.to_string())
                .fold(String::with_capacity(line.len() * 4), |str, el| str + &el + " ");

            // Cap the line off with a newline
            if line_string.pop().is_some(){
                line_string.push('\n');
            }

            image_string.push_str(&line_string);
        }

        // Write whole image
        writer.write_all(image_string.as_bytes())?;

        Ok(())
    }

    fn gamma_correct(self) -> Self {
        Self {
            r: self.r().powf(1. / 2.2),
            g: self.g().powf(1. / 2.2),
            b: self.b().powf(1. / 2.2),
        }
    }

    pub fn random(rand: &mut XorShift) -> Self {
        Self {
            r: rand.next_01(),
            g: rand.next_01(),
            b: rand.next_01(),
        }
    }

    #[allow(clippy::unused_self)]
    pub const fn set_red(self) -> Self {
        Self {
            r: 1.,
            g: 0.,
            b: 0.,
        }
    }
}
