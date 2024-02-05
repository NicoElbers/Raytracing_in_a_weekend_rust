use crate::space::point3::Point3;
use crate::space::vec3::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Ray {
    #[must_use]
    pub const fn new(point: Point3, vec: Vec3) -> Self {
        Self {
            orig: point,
            dir: vec,
        }
    }

    #[must_use]
    pub fn at(&self, t: f64) -> Point3 {
        let ray_point: Point3 = (self.dir() * t).into();
        ray_point + self.orig()
    }

    #[must_use]
    pub const fn orig(&self) -> Point3 {
        self.orig
    }

    #[must_use]
    pub const fn dir(&self) -> Vec3 {
        self.dir
    }

    #[must_use]
    pub fn offset(self, offset: &Vec3) -> Self {
        let dir = self.dir + *offset;
        let dir = dir.unit();

        Self {
            orig: self.orig,
            dir,
        }
    }
}
