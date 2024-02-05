use std::rc::Rc;

use crate::raytracing::hittable::HitRecord;
use crate::raytracing::hittable::Hittable;
use crate::raytracing::materials::Material;
use crate::raytracing::ray::Ray;
use crate::space::point3::Point3;
use crate::space::vec3::Vec3;
use crate::util::interval::Interval;

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Rc<dyn Material>,
}

impl Sphere {
    #[must_use]
    pub const fn new(center: Point3, radius: f64, mat: Rc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            mat,
        }
    }

    #[must_use]
    pub fn new_world_obj(x: f64, y: f64, z: f64, radius: f64, mat: Rc<dyn Material>) -> Rc<Self> {
        let center = Point3::new(x, y, z);
        Rc::new(Self {
            center,
            radius,
            mat,
        })
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, inter: &Interval) -> Option<HitRecord> {
        // Original center, or something, idrk
        let oc = r.orig() - self.center;
        let oc: Vec3 = oc.into();

        // Quadratic formula
        // TODO: Think of factoring this out, might be used a lot
        let a = r.dir().len_squared(); // Dot product with self
        let half_b = Vec3::dot(oc, r.dir());
        let c = oc.len_squared() - self.radius * self.radius;

        #[allow(clippy::suspicious_operation_groupings)]
        let d = (half_b * half_b) - (a * c);

        // If discriminant < 0, then no hit
        if d < 0.0 {
            return None;
        }

        let sqrtd: f64 = d.sqrt();

        let root: f64 = [-sqrtd, sqrtd]
            .into_iter()
            .map(|x| (x - half_b) / a)
            .find(|x| inter.contains_inc(*x))?;

        let point = r.at(root);
        let normal = (point - self.center) / self.radius;
        let normal: Vec3 = normal.into();

        Some(HitRecord::new(point, normal, root, *r, self.mat.clone()))
    }
}
