mod color;
mod ray;
mod vec3;

use std::{fs::File, io::Write};

use crate::color::Color;
use crate::ray::Point3;
use crate::ray::Ray;
use crate::vec3::Vec3;

fn ray_color(ray: Ray) -> Color {
    let unit_dir = ray.dir().unit();
    let a = 0.5 * (unit_dir.y() + 1.0);

    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

fn main() -> std::io::Result<()> {
    let aspect_ratio = 16.0 / 9.0;

    // Image size
    let image_height: u64 = 2160;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let image_width: u64 = (image_height as f64 * aspect_ratio) as u64;

    // Camera
    let focal_length = 1.0;
    let viewport_height = 2.0;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let cam = Point3::default();

    // Help vectors
    let vp_u = Vec3::new(viewport_width, 0.0, 0.0);
    let vp_v = Vec3::new(0.0, -viewport_height, 0.0);

    // Calculate deltas
    #[allow(clippy::cast_precision_loss)]
    let pixel_delta_u = vp_u / image_width as f64;
    #[allow(clippy::cast_precision_loss)]
    let pixel_delta_v = vp_v / image_height as f64;

    // Calculate first pixel
    let vp_upper_left = cam - Vec3::new(0.0, 0.0, focal_length) - vp_u / 2.0 - vp_v / 2.0;
    let pixel00 = vp_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

    // Get file
    let mut file = File::create("img.ppm")?;

    println!("Making an image of format:\n\t{image_width} by {image_height}");

    // Renderer
    write!(file, "P3\n{image_width} {image_height}\n255\n")?;

    let mut last_prog: u64 = 0;
    for j in 0..image_height {
        // Progress
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let prog = ((j as f64 / image_height as f64) * 100.0) as u64;
        if last_prog != prog {
            println!("[ INFO ] {prog}% done");
            last_prog = prog;
        }

        for i in 0..image_width {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let pixel_center = pixel00 + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_dir = pixel_center - cam;
            let ray_dir = ray_dir.unit();

            let ray = Ray::new(cam, ray_dir);

            let color = ray_color(ray);

            color.write(&mut file)?;
        }
    }

    Ok(())
}
