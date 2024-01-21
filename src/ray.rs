use crate::point3::Point3;
use crate::vec3::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Ray {
    pub const fn new(point: Point3, vec: Vec3) -> Self {
        Self {
            orig: point,
            dir: vec,
        }
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.dir * t + self.orig
    }

    pub const fn orig(&self) -> Point3 {
        self.orig
    }

    pub const fn dir(&self) -> Vec3 {
        self.dir
    }
}
