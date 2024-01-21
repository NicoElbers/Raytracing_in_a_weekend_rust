mod color;
mod hittable;
mod point3;
mod ray;
mod shapes;
mod vec3;

use std::{fs::File, io::Write};

use hittable::Hittable;

use crate::color::Color;
use crate::hittable::List;
use crate::point3::Point3;
use crate::ray::Ray;
use crate::shapes::Sphere;
use crate::vec3::Vec3;

fn ray_color(ray: Ray, obj: &dyn Hittable) -> Color {
    if let Some(record) = obj.hit(&ray, 0.0, f64::INFINITY) {
        let color_normal: Color = record.normal().into();
        return 0.5 * (color_normal + Color::new(1.0, 1.0, 1.0));
    }

    let unit_dir = ray.dir().unit();
    let a = 0.5 * (unit_dir.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

fn main() -> std::io::Result<()> {
    const ASPECT_RATIO: f64 = 16.0 / 9.0;

    // Image size
    const IMAGE_HEIGHT: u64 = 2160;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    const IMAGE_WIDTH: u64 = (IMAGE_HEIGHT as f64 * ASPECT_RATIO) as u64;

    // Camera
    const FOCAL_LENGTH: f64 = 1.0;
    const VIEWPORT_HEIGHT: f64 = 2.0;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    const VIEWPORT_WIDTH: f64 = VIEWPORT_HEIGHT * (IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64);
    let cam = Point3::default();

    // Help vectors
    let vp_u = Vec3::new(VIEWPORT_WIDTH, 0.0, 0.0);
    let vp_v = Vec3::new(0.0, -VIEWPORT_HEIGHT, 0.0);

    // Calculate deltas
    #[allow(clippy::cast_precision_loss)]
    let pixel_delta_u = vp_u / IMAGE_WIDTH as f64;
    #[allow(clippy::cast_precision_loss)]
    let pixel_delta_v = vp_v / IMAGE_HEIGHT as f64;

    // Calculate first pixel
    let vp_upper_left = cam - Vec3::new(0.0, 0.0, FOCAL_LENGTH) - vp_u / 2.0 - vp_v / 2.0;
    let pixel00 = vp_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

    // World elements
    let mut world = List::new();

    world.add(Sphere::new_world_obj(0.0, 0.0, -2.0, 1.4));
    world.add(Sphere::new_world_obj(0.0, 0.0, -1.0, 0.5));
    world.add(Sphere::new_world_obj(0.0, -105.0, -1.0, 100.0));

    // Get file
    let mut file = File::create("img.ppm")?;

    println!("Making an image of format:\n\t{IMAGE_WIDTH} by {IMAGE_HEIGHT}");

    // Renderer
    write!(file, "P3\n{IMAGE_WIDTH} {IMAGE_HEIGHT}\n255\n")?;

    let mut last_prog: u64 = 0;
    for j in 0..IMAGE_HEIGHT {
        // Progress
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let prog = ((j as f64 / IMAGE_HEIGHT as f64) * 100.0) as u64;
        if last_prog != prog {
            println!("[ INFO ] {prog}% done");
            last_prog = prog;
        }

        for i in 0..IMAGE_WIDTH {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let pixel_center = pixel00 + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_dir: Vec3 = (pixel_center - cam).into();
            let ray_dir = ray_dir.unit();

            let ray = Ray::new(cam, ray_dir);

            let color = ray_color(ray, &world);

            color.write(&mut file)?;
        }
    }

    Ok(())
}
