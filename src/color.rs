use std::{fs::File, io::Write};

use crate::vec3::Vec3;

pub type Color = Vec3;

impl Color {
    pub fn r(&self) -> f64 {
        self.x()
    }

    pub fn g(&self) -> f64 {
        self.y()
    }

    pub fn b(&self) -> f64 {
        self.z()
    }

    pub fn write(&self, file: &mut File) -> std::io::Result<()> {
        let colors = *self * 255.0;

        assert!(colors.r() <= 255.0);
        assert!(colors.r() >= 0.0);
        assert!(colors.g() <= 255.0);
        assert!(colors.g() >= 0.0);
        assert!(colors.b() <= 255.0);
        assert!(colors.b() >= 0.0);

        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
        )]
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
