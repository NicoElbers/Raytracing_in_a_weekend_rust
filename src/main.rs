// TODO: Factor all the raytrace code out to a module
// TODO: Implement a winit application to show progress while we're rendering
// TODO: Render to png instead of ppm
// TODO: Make this motherfucker a beast by implementing WebGPU rendering

mod raytracing;
mod space;
mod util;

use std::error::Error;
use std::sync::Arc;

use raytracing::camera::Camera;
use raytracing::color::Color;
use raytracing::hittable::{Hittable, SceneBuilder};
use raytracing::materials::{Dielectric, Lambertian, Material, Metal};
use raytracing::shapes::sphere::Sphere;
use space::point3::Point3;
use space::vec3::Vec3;
use util::random::XorShift;

const ASPECT_RATIO: f64 = 16.0 / 9.0;

// Image size
const IMAGE_HEIGHT: usize = 1440;
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
const IMAGE_WIDTH: usize = (IMAGE_HEIGHT as f64 * ASPECT_RATIO) as usize;

// Camera
const FOCAL_LENGTH: f64 = 1.0;

const FOV: f64 = 20.;

const SAMPLE_SQRT: usize = 45;
const MAX_DEPTH: usize = 50;

const LOOK_FROM: Point3 = Point3::new(13., 2., 3.);
const LOOK_TO: Point3 = Point3::new(0., 0., 0.);

const VUP: Vec3 = Vec3::new(0., 1., 0.);

const DEFOCUS_ANGLE: f64 = 0.6;
const FOCUS_DIST: f64 = 10.0;

fn main() -> Result<(), Box<dyn Error>> {
    // simple()?;
    complex()?;
    // super_simple()?;
    // threads()?;

    Ok(())
}

#[allow(dead_code)]
fn complex() -> Result<(), Box<dyn Error>> {
    let mut world = SceneBuilder::new();

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Sphere::new_world_obj(0., -1000., 0., 1000., ground_mat));

    let mut rand = XorShift::default();

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand.next_01();
            let center = Point3::new(
                f64::from(a) + 0.9 * rand.next_01(),
                0.2,
                f64::from(b) + 0.9 * rand.next_01(),
            );

            let point_vec: Vec3 = (center - Point3::new(4., 0.2, 0.)).into();
            if point_vec.len() > 0.9 {
                let mat: Arc<dyn Material> = if choose_mat < 0.34 {
                    let albedo = Color::random(&mut rand) * Color::random(&mut rand);
                    Arc::new(Lambertian::new(albedo))
                } else if choose_mat < 0.67 {
                    let albedo = Color::random(&mut rand) * Color::random(&mut rand);
                    let fuzz = rand.next_bound(0., 1.);
                    Arc::new(Metal::new(albedo, fuzz))
                } else {
                    Arc::new(Dielectric::new(1.5))
                };

                world.add(Arc::new(Sphere::new(center, 0.2, mat)));
            }
        }
    }

    let glass = Arc::new(Dielectric::new(1.5));
    world.add(Sphere::new_world_obj(0., 1., 0., 1., glass));

    let diffuse = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Sphere::new_world_obj(-4., 1., 0., 1., diffuse));

    let metal = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Sphere::new_world_obj(4., 1., 0., 1., metal));

    let cam = Camera::new(
        IMAGE_HEIGHT,
        IMAGE_WIDTH,
        MAX_DEPTH,
        FOCAL_LENGTH,
        FOV,
        LOOK_FROM,
        LOOK_TO,
        VUP,
        DEFOCUS_ANGLE,
        FOCUS_DIST,
    );

    let world = world.build() as Arc<dyn Hittable>;
    // cam.render(&world, SAMPLE_SQRT)?;

    let cam = Arc::new(cam);
    Camera::threaded_render(&cam, &world, SAMPLE_SQRT)?;

    Ok(())
}

#[allow(dead_code)]
fn simple() -> Result<(), Box<dyn Error>> {
    let cam = Camera::new(
        1080,
        1920,
        25,
        1.0,
        20.0,
        Point3::new(-2., 2., 1.),
        Point3::new(0., 0., -1.),
        Vec3::new(0., 1., 0.),
        10.0,
        3.4,
    );

    // Materials
    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_left = Arc::new(Dielectric::new(1.5));
    let mat_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.));

    // World elements
    let mut world = SceneBuilder::new();

    // world.add(Sphere::new_world_obj(0., 0., -5., 1.4));
    // world.add(Sphere::new_world_obj(0., 0., -1., 0.2));
    // world.add(Sphere::new_world_obj(-1., 1., -1.2, 0.5));
    // world.add(Sphere::new_world_obj(0., -105., 0., 100.));

    world.add(Sphere::new_world_obj(0., -100.5, -1., 100., mat_ground));
    world.add(Sphere::new_world_obj(0., 0., -1., 0.5, mat_center));
    world.add(Sphere::new_world_obj(-1., 0., -1., 0.5, mat_left));
    world.add(Sphere::new_world_obj(1., 0., -1., 0.5, mat_right));

    // world.add(Sphere::new_world_obj(-r, 0., -1., r, mat_left));
    // world.add(Sphere::new_world_obj(r, 0., -1., r, mat_right));

    let world = world.build() as Arc<dyn Hittable>;
    // cam.render(&world, SAMPLE_SQRT)?;

    let cam = Arc::new(cam);
    Camera::threaded_render(&cam, &world, SAMPLE_SQRT)?;

    Ok(())
}

#[allow(dead_code)]
fn threads() -> Result<(), Box<dyn Error>> {
    let cam = Camera::new(
        1000,
        1000,
        50,
        1.0,
        50.0,
        Point3::new(0., 0., 0.),
        Point3::new(0., 0., -0.3),
        Vec3::new(0., 1., 0.),
        0.6,
        10.0,
    );

    let cam = Arc::new(cam);

    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    let mut world = SceneBuilder::new();
    world.add(Sphere::new_world_obj(0., -100.5, -1., 100., mat_ground));
    let world = world.build() as Arc<dyn Hittable>;

    Camera::threaded_render(&cam, &world, SAMPLE_SQRT)?;

    Ok(())
}

#[allow(dead_code)]
fn super_simple() -> Result<(), Box<dyn Error>> {
    let cam = Camera::new(
        1000,
        1000,
        50,
        1.0,
        50.0,
        Point3::new(0., 0., 0.),
        Point3::new(0., 0., -0.3),
        Vec3::new(0., 1., 0.),
        0.6,
        10.0,
    );
    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    // World elements
    let mut world = SceneBuilder::new();
    world.add(Sphere::new_world_obj(0., -100.5, -1., 100., mat_ground));
    let world = world.build();

    let obj = world.objects();
    assert!(!obj.is_empty());

    let world = world as Arc<dyn Hittable>;
    // cam.render(&world, SAMPLE_SQRT)?;

    let cam = Arc::new(cam);
    Camera::threaded_render(&cam, &world, SAMPLE_SQRT)?;

    Ok(())
}
