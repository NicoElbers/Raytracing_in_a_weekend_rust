use std::cmp::Ordering;
use std::fmt::Debug;
use std::sync::Arc;

use crate::raytracing::materials::Material;
use crate::raytracing::ray::Ray;
use crate::space::point3::Point3;
use crate::space::vec3::Vec3;
use crate::util::interval::Interval;

pub trait Hittable: Debug + Send + Sync {
    fn hit(&self, r: &Ray, inter: &Interval) -> Option<HitRecord>;
}

#[derive(Debug, Clone)]
pub struct HitRecord {
    point: Point3,
    normal: Vec3,
    mat: Arc<dyn Material>,
    time: f64,
    front_face: bool,
}

impl HitRecord {
    #[must_use]
    pub fn new(point: Point3, normal: Vec3, time: f64, ray: Ray, mat: Arc<dyn Material>) -> Self {
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
    #[allow(dead_code)]
    pub const fn time(&self) -> f64 {
        self.time
    }

    #[must_use]
    pub const fn front_face(&self) -> bool {
        self.front_face
    }

    pub fn mat(&self) -> Arc<dyn Material> {
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

#[derive(Debug, Default, Clone)]
pub struct SceneBuilder {
    objects: Vec<Arc<SceneObject>>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn build(self) -> Arc<Scene> {
        if self.objects.is_empty() {
            let non_empty_objects: Vec<Arc<SceneObject>> = vec![Arc::new(Empty::default())];

            debug_assert!(!non_empty_objects.is_empty());

            Arc::new(Scene {
                objects: non_empty_objects,
            })
        } else {
            Arc::new(Scene {
                objects: self.objects,
            })
        }
    }

    pub fn add(&mut self, obj: Arc<SceneObject>) {
        self.objects.push(obj);
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    objects: Vec<Arc<SceneObject>>,
}

impl Scene {
    #[must_use]
    pub fn objects(&self) -> &Vec<Arc<SceneObject>> {
        &self.objects
    }
}

impl Hittable for Scene {
    fn hit(&self, ray: &Ray, inter: &Interval) -> Option<HitRecord> {
        debug_assert!(!self.objects.is_empty(), "Cannot hit if scene is empty");

        self.objects
            .iter()
            .filter_map(|obj| obj.hit(ray, inter))
            .min_by(|closest, next| {
                f64::partial_cmp(&closest.time, &next.time) //
                    .unwrap_or(Ordering::Equal)
            })
    }
}

#[derive(Debug, Default)]
pub struct Empty {}

impl Hittable for Empty {
    fn hit(&self, _r: &Ray, _inter: &Interval) -> Option<HitRecord> {
        None
    }
}
