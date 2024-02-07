use std::fmt::Debug;

use crate::{space::vec3::Vec3, util::random::XorShift};

use super::{color::Color, hittable::HitRecord, ray::Ray};

pub trait Material: Debug + Sync + Send {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rand: &mut XorShift) -> Option<(Ray, Color)>;
}

#[derive(Debug)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub const fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, record: &HitRecord, rand: &mut XorShift) -> Option<(Ray, Color)> {
        let scatter_dir = record.normal() + Vec3::random_unit_vec(rand);

        let scatter_dir = if scatter_dir.near_zero() {
            record.normal()
        } else {
            scatter_dir
        };

        let scattered = Ray::new(record.point(), scatter_dir);
        let color = self.albedo;

        Some((scattered, color))
    }
}

#[derive(Debug)]
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        assert!(fuzz <= 1., "Fuzz cannot be more than 1");
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rand: &mut XorShift) -> Option<(Ray, Color)> {
        let reflected = ray.dir().unit().reflect(&record.normal());
        let scattered = Ray::new(
            record.point(),
            reflected + self.fuzz * Vec3::random_unit_vec(rand),
        );
        let color = self.albedo;

        Some((scattered, color))
    }
}

#[derive(Debug)]
pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    pub const fn new(ir: f64) -> Self {
        Self { ir }
    }

    fn reflectance(&self, cos: f64) -> f64 {
        let ir = self.ir;
        let r0 = (1. - ir) / (1. + ir);
        let r0 = r0 * r0;
        r0 + (1. - r0) * (1. - cos).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rand: &mut XorShift) -> Option<(Ray, Color)> {
        let refraction_ratio = if record.front_face() {
            1. / self.ir
        } else {
            self.ir
        };

        let unit_direction = ray.dir().unit();

        let cos_theta = f64::min(Vec3::dot(-unit_direction, record.normal()), 1.);
        let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);

        let cant_refract = refraction_ratio * sin_theta > 1.;

        let direction = if cant_refract || self.reflectance(cos_theta) > rand.next_01() {
            // Cannot refract
            unit_direction.reflect(&record.normal())
        } else {
            // may refact
            unit_direction.refract(&record.normal(), refraction_ratio)
        };

        let scattered = Ray::new(record.point(), direction);
        let color = Color::new(1., 1., 1.);

        Some((scattered, color))
    }
}
