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
        let d = half_b * half_b - a * c;

        // If discriminant < 0, then no hit
        if d < 0.0 {
            return None;
        }

        let sqrt = d.sqrt();

        // Quite happy with this
        let root = match sqrt {
            root if (ray_tmin..=ray_tmax).contains(&((-half_b - sqrt) / a)) => root,
            root if (ray_tmin..=ray_tmax).contains(&((-half_b + sqrt) / a)) => root,
            _ => return None,
        };

        let point = r.at(root);
        let normal = (point - self.center) / self.radius;
        let normal: Vec3 = normal.into();
        Some(HitRecord::new(point, normal, root, *r))
    }
}
