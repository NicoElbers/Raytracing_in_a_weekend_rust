use std::rc::Rc;

use crate::{
    hittable::{HitRecord, Hittable},
    point3::Point3,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Sphere {
    center: Point3,
    radius: f64,
}

impl Sphere {
    pub const fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn new_world_obj(x: f64, y: f64, z: f64, radius: f64) -> Rc<Self> {
        let point = Point3::new(x, y, z);
        Rc::new(Self {
            center: point,
            radius,
        })
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64) -> Option<HitRecord> {
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

        let sqrtd = d.sqrt();

        // Quite happy with this
        let root = match sqrtd {
            sqrt if (ray_tmin..=ray_tmax).contains(&((-half_b - sqrtd) / a)) => {
                (-half_b - sqrt) / a
            }
            sqrt if (ray_tmin..=ray_tmax).contains(&((-half_b + sqrtd) / a)) => {
                (-half_b + sqrt) / a
            }
            _ => return None,
        };

        let point = r.at(root);
        let normal = (point - self.center) / self.radius;
        let normal: Vec3 = normal.into();

        // dbg!(root);
        // debug_assert!(normal.is_unit(), "normal: {}, root {}", normal, root);

        Some(HitRecord::new(point, normal, root, *r))
    }
}
