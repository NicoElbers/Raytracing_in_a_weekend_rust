use crate::vec3::Vec3;

pub type Point3 = Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Ray {
    pub fn new(point: Point3, vec: Vec3) -> Self {
        Self {
            orig: point,
            dir: vec,
        }
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.orig + self.dir * t
    }

    pub fn orig(&self) -> Vec3 {
        self.orig
    }

    pub fn dir(&self) -> Vec3 {
        self.dir
    }
}
