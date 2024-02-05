use std::cmp::Ordering;
use std::fmt::Debug;
use std::rc::Rc;

use crate::raytracing::materials::Material;
use crate::raytracing::ray::Ray;
use crate::space::point3::Point3;
use crate::space::vec3::Vec3;
use crate::util::interval::Interval;

pub trait Hittable: Debug {
    fn hit(&self, r: &Ray, inter: &Interval) -> Option<HitRecord>;
}

#[derive(Debug, Clone)]
pub struct HitRecord {
    point: Point3,
    normal: Vec3,
    mat: Rc<dyn Material>,
    time: f64,
    front_face: bool,
}

impl HitRecord {
    #[must_use]
    pub fn new(point: Point3, normal: Vec3, time: f64, ray: Ray, mat: Rc<dyn Material>) -> Self {
        let (front_face, normal) = Self::face_normal(&ray, &normal);

        Self {
            point,
            normal,
            mat,
            time,
            front_face,
        }
    }

    #[must_use]
    pub const fn point(&self) -> Point3 {
        self.point
    }

    #[must_use]
    pub const fn normal(&self) -> Vec3 {
        self.normal
    }

    #[must_use]
    pub const fn time(&self) -> f64 {
        self.time
    }

    #[must_use]
    pub const fn front_face(&self) -> bool {
        self.front_face
    }

    pub fn mat(&self) -> Rc<dyn Material> {
        self.mat.clone()
    }

    fn face_normal(r: &Ray, outward_normal: &Vec3) -> (bool, Vec3) {
        debug_assert!(
            outward_normal.is_unit(0.1),
            "Outward normal len is {}\nfull vector {}",
            outward_normal.len(),
            outward_normal
        );

        let outward_normal = *outward_normal;

        let front_face = Vec3::dot(r.dir(), outward_normal) < 0.0;

        if front_face {
            (true, outward_normal)
        } else {
            (false, -outward_normal)
        }
    }
}

pub type SceneObject = dyn Hittable;

#[derive(Default, Debug, Clone)]
pub struct Scene {
    objects: Vec<Rc<SceneObject>>,
}

impl Scene {
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    #[must_use]
    pub fn objects(&self) -> &Vec<Rc<SceneObject>> {
        &self.objects
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, obj: Rc<SceneObject>) {
        self.objects.push(obj);
    }
}

impl Hittable for Scene {
    fn hit(&self, ray: &Ray, inter: &Interval) -> Option<HitRecord> {
        self.objects
            .iter()
            .filter_map(|obj| obj.hit(ray, inter))
            .min_by(|closest, next| {
                f64::partial_cmp(&closest.time, &next.time) //
                    .unwrap_or(Ordering::Equal)
            })
    }
}
