use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::point3::Point3;

#[derive(Default, Debug, Clone, Copy)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
}

trait Hittable {
    fn new(p: Point3, normal: Vec3, t: f64) -> Self;
    fn hit(r: &Ray, ray_tmin: f64, rec: HitRecord) -> bool;
}
