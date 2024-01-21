use std::cmp::Ordering;
use std::fmt::Debug;
use std::rc::Rc;

use crate::point3::Point3;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct HitRecord {
    point: Point3,
    normal: Vec3,
    time: f64,
    front_face: bool,
}

impl HitRecord {
    pub fn new(point: Point3, normal: Vec3, time: f64, ray: Ray) -> Self {
        let (front_face, normal) = Self::face_normal(ray, normal);

        Self {
            point,
            normal,
            time,
            front_face,
        }
    }

    pub const fn point(&self) -> Point3 {
        self.point
    }

    pub const fn normal(&self) -> Vec3 {
        self.normal
    }

    pub const fn time(&self) -> f64 {
        self.time
    }

    pub const fn front_face(&self) -> bool {
        self.front_face
    }

    fn face_normal(r: Ray, outward_normal: Vec3) -> (bool, Vec3) {
        #[cfg(debug_assertions)]
        assert!(outward_normal.is_unit());

        let front_face = Vec3::dot(r.dir(), outward_normal) < 0.0;

        if front_face {
            (true, outward_normal)
        } else {
            (false, -outward_normal)
        }
    }
}

pub trait Hittable: Debug {
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64) -> Option<HitRecord>;
}

#[derive(Default, Debug, Clone)]
pub struct List {
    objects: Vec<Rc<dyn Hittable>>,
}

impl List {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn objects(&self) -> &Vec<Rc<dyn Hittable>> {
        &self.objects
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, obj: Rc<dyn Hittable>) {
        self.objects.push(obj);
    }

    pub fn hit_closest(&self, ray: &Ray, ray_tmin: f64, ray_tmax: f64) -> Option<HitRecord> {
        self.objects
            .iter()
            .filter_map(|obj| obj.hit(ray, ray_tmin, ray_tmax))
            .min_by(|closest, next| {
                f64::partial_cmp(&closest.time, &next.time) //
                    .unwrap_or(Ordering::Equal)
            })
    }
}
