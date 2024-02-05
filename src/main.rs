// TODO: FUCKING FINISH THE FUCKER
// TODO: Factor all the raytrace code out to a module
// TODO: Implement a winit application to show progress while we're rendering
// TODO: Render to png instead of ppm
// TODO: Make this motherfucker a beast by implementing WebGPU rendering

mod raytracing;
mod space;
mod util;

use std::f64::consts::PI;
use std::rc::Rc;

use crate::raytracing::shapes::sphere::Sphere;
use raytracing::camera::Camera;
use raytracing::color::Color;
use raytracing::hittable::Scene;
use raytracing::materials::{Dielectric, Lambertian, Metal};
use space::point3::Point3;

const ASPECT_RATIO: f64 = 16.0 / 9.0;

// Image size
const IMAGE_HEIGHT: u64 = 400;
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
const IMAGE_WIDTH: u64 = (IMAGE_HEIGHT as f64 * ASPECT_RATIO) as u64;

// Camera
const FOCAL_LENGTH: f64 = 1.0;
const VIEWPORT_HEIGHT: f64 = 2.0;

const FOV: f64 = 90.;

const SAMPLE_SQRT: usize = 10;
const MAX_DEPTH: usize = 50;

fn main() -> std::io::Result<()> {
    let cam = Camera::new(
        IMAGE_HEIGHT,
        IMAGE_WIDTH,
        MAX_DEPTH,
        FOCAL_LENGTH,
        FOV,
        Point3::default(),
    );

    // Materials
    let mat_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    // let mat_center = Rc::new(Lambertian::new(Color::new(0.7, 0.3, 0.3)));
    let mat_center = Rc::new(Dielectric::new(1.5));
    let mat_left = Rc::new(Lambertian::new(Color::new(0., 0., 1.)));
    let mat_right = Rc::new(Lambertian::new(Color::new(1., 0., 0.)));

    let r = f64::cos(PI / 4.);

    // World elements
    let mut world = Scene::new();

    // world.add(Sphere::new_world_obj(0., 0., -5., 1.4));
    // world.add(Sphere::new_world_obj(0., 0., -1., 0.2));
    // world.add(Sphere::new_world_obj(-1., 1., -1.2, 0.5));
    // world.add(Sphere::new_world_obj(0., -105., 0., 100.));

    // world.add(Sphere::new_world_obj(0., -100.5, -1., 100., mat_ground));
    // world.add(Sphere::new_world_obj(0., 0., -1., -0.5, mat_center));
    // world.add(Sphere::new_world_obj(-1., 0., -1., 0.5, mat_left));
    // world.add(Sphere::new_world_obj(1., 0., -1., 0.5, mat_right));

    world.add(Sphere::new_world_obj(-r, 0., -1., r, mat_left));
    world.add(Sphere::new_world_obj(r, 0., -1., r, mat_right));

    cam.render(&world, SAMPLE_SQRT)?;

    Ok(())
}
