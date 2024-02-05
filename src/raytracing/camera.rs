use std::fs::File;
use std::io::{stdout, BufWriter, Write};

use crate::raytracing::color::Color;
use crate::raytracing::hittable::SceneObject;
use crate::raytracing::ray::Ray;
use crate::space::point3::Point3;
use crate::space::vec3::Vec3;
use crate::util::interval::Interval;
use crate::util::random::XorShift;

#[derive(Default, Debug, Clone, Copy)]
/// [Camera] stores information for a camera in a scene. It sets up a location
/// and some screen information (aspect ratio, height, focal length, etc).
pub struct Camera {
    img: ImgData,
    cam: CamData,
    // NOTE: pixel00 is equal to the top left of the viewport. Since a lattice
    // of offsets is used, they are offset from the top left of a pixel
    pixel00: Point3,
    max_depth: usize,

    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    #[must_use]
    pub fn new(
        img_height: u64,
        img_width: u64,
        max_depth: usize,
        focal_length: f64,
        fov: f64,
        center: Point3,
    ) -> Self {
        // Viewport
        let theta = f64::to_radians(fov);
        let h = f64::tan(theta / 2.);
        let viewport_height: f64 = 2. * h * focal_length;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let viewport_width: f64 = viewport_height * (img_width as f64 / img_height as f64);

        // Viewport vectors
        let vp_u = Vec3::new(viewport_width, 0.0, 0.0);
        let vp_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate deltas
        #[allow(clippy::cast_precision_loss)]
        let pixel_delta_u = vp_u / img_width as f64;
        #[allow(clippy::cast_precision_loss)]
        let pixel_delta_v = vp_v / img_height as f64;

        // Calculate first pixel, this is the upper left of the viewport
        // Intentionally NOT the middle of the pixel
        let pixel00 = center - Vec3::new(0.0, 0.0, focal_length) - vp_u / 2.0 - vp_v / 2.0;

        // Data helper structs
        let cam = CamData::new(focal_length, viewport_height, viewport_width, fov, center);
        let img = ImgData::new(img_height, img_width);

        Self {
            img,
            cam,
            pixel00,
            max_depth,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    /// Render the world as displayed by the world parameter. It will render all
    /// objects
    ///
    /// # Errors
    ///
    /// This function will return an error if it cannot open a file called img.ppm
    /// or if it can later not write to that file.
    pub fn render(&self, world: &SceneObject, samples_sqrt: usize) -> std::io::Result<()> {
        // Get file
        let file = File::create("img.ppm")?;
        let mut writer = BufWriter::new(&file);

        let offsets: Vec<Vec3> =
            Self::offset_lattice(&self.pixel_delta_v, &self.pixel_delta_u, samples_sqrt);

        println!(
            "Making an image of format:\n\t{} by {}\n\t{} samples\n\t{} max depth",
            self.img.width,
            self.img.height,
            samples_sqrt * samples_sqrt,
            self.max_depth,
        );

        // Renderer
        write!(writer, "P3\n{} {}\n255\n", self.img.width, self.img.height)?;

        let mut rand = XorShift::default();

        let mut last_prog: u64 = 0;
        let mut out = stdout();
        // out.write_all(b"\r\x1b[2K")?;
        print!("[ INFO ] 0% done");
        out.flush()?;
        for j in 0..self.img.height {
            // Progress
            // TODO: Factor this out, this is unreadable
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let prog = ((j as f64 / self.img.height as f64) * 100.0) as u64;
            if last_prog != prog {
                let mut out = stdout();
                out.write_all(b"\r\x1b[2K")?;
                print!("[ INFO ] {prog}% done");
                out.flush()?;
                last_prog = prog;
            }

            for i in 0..self.img.width {
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                let pixel_center = self.pixel00
                    + (i as f64 * self.pixel_delta_u)
                    + (j as f64 * self.pixel_delta_v);
                let ray_dir: Vec3 = (pixel_center - self.cam.center).into();
                let ray_dir = ray_dir.unit();

                let ray = Ray::new(self.cam.center, ray_dir);

                // let avg_color = Self::ray_colors_random(&ray, world, &mut random, samples_sqrt, &self.pixel_delta_u, &self.pixel_delta_v);
                let avg_color = Self::ray_colors_lattice(self, ray, world, &offsets, &mut rand);

                avg_color.write(&mut writer)?;
            }
        }
        writer.flush()?;

        let mut out = stdout();
        out.write_all(b"\r\x1b[2K")?;
        println!("[ INFO ]     done!");
        out.flush()?;

        Ok(())
    }

    #[allow(dead_code)]
    fn ray_colors_lattice(
        &self,
        ray: Ray,
        world: &SceneObject,
        offsets: &[Vec3],
        rand: &mut XorShift,
    ) -> Color {
        #[allow(clippy::cast_precision_loss)]
        let avg_color = offsets
            .iter()
            .map(|offset| ray.offset(offset))
            .map(|ray| Self::ray_color(self, ray, world, rand, 0))
            .fold(Color::default(), |acc, ray| acc + ray)
            / offsets.len() as f64;
        avg_color
    }

    #[allow(dead_code)]
    fn ray_colors_random(
        &self,
        ray: Ray,
        world: &SceneObject,
        rand: &mut XorShift,
        samples: usize,
        dx: &Vec3,
        dy: &Vec3,
    ) -> Color {
        let mut total_color = Color::default();
        for _ in 0..samples {
            let ray = ray.offset(&(*dx * rand.next_01() + *dy * rand.next_01()));
            let color = Self::ray_color(self, ray, world, rand, 0);
            total_color = total_color + color;
        }

        #[allow(clippy::cast_precision_loss)]
        let avg_color = total_color / samples as f64;

        avg_color
    }

    fn ray_color(&self, ray: Ray, obj: &SceneObject, rand: &mut XorShift, depth: usize) -> Color {
        if depth > self.max_depth {
            return Color::new(0., 0., 0.);
        }

        if let Some(record) = obj.hit(&ray, &Interval::from(0.01)) {
            // Random, naive method
            // let new_direction = Vec3::random_vec_on_hemishpere(rand, &record.normal());
            // Lambertian reflection, correct (and faster) method
            // let new_direction = record.normal() + Vec3::random_unit_vec(rand);

            if let Some((ray, color)) = record.mat().scatter(&ray, &record, rand) {
                return color * self.ray_color(ray, obj, rand, depth + 1);
            }

            return Color::default();
        }

        let unit_dir = ray.dir().unit();
        let a = 0.5 * (unit_dir.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    fn offset_lattice(dx: &Vec3, dy: &Vec3, num_layers: usize) -> Vec<Vec3> {
        if num_layers == 0 {
            return vec![*dx / 2. + *dy / 2.];
        }

        #[allow(clippy::cast_precision_loss)]
        let num_layers_f64 = num_layers as f64;

        let dx = *dx / num_layers_f64;
        let dy = *dy / num_layers_f64;

        let pos0 = dx / 2. + dy / 2.;

        let mut offsets: Vec<Vec3> = Vec::with_capacity(num_layers * num_layers);

        for y in 0..num_layers {
            // NOTE: We add because dy will be negative in this program. So adding
            // dy will go down
            #[allow(clippy::cast_precision_loss)]
            let pos = pos0 + dy * y as f64;
            for x in 0..num_layers {
                #[allow(clippy::cast_precision_loss)]
                let pos = pos + dx * x as f64;
                offsets.push(pos);
            }
        }

        offsets
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct ImgData {
    pub height: u64,
    pub width: u64,
}

impl ImgData {
    const fn new(img_height: u64, img_width: u64) -> Self {
        Self {
            height: img_height,
            width: img_width,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct CamData {
    #[allow(dead_code)]
    pub focal_length: f64,
    #[allow(dead_code)]
    pub viewport_height: f64,
    #[allow(dead_code)]
    pub viewport_width: f64,
    pub fov: f64,
    pub center: Point3,
}

impl CamData {
    const fn new(
        focal_length: f64,
        viewport_height: f64,
        viewport_width: f64,
        fov: f64,
        center: Point3,
    ) -> Self {
        Self {
            focal_length,
            viewport_height,
            viewport_width,
            fov,
            center,
        }
    }
}

#[cfg(test)]
mod camera_tests {
    use crate::raytracing::camera::{Camera, Vec3};

    // NOTE: This test ouputs points that can be thrown into desmos, run it using
    // `cargo test -- --nocapture`
    #[test]
    fn display_offsets() {
        let dx = Vec3::new(1., 0., -1.);
        let dy = Vec3::new(0., -1., 0.);

        let dx = dx.unit();
        let dy = dy.unit();

        let layer0 = Camera::offset_lattice(&dx, &dy, 0);
        assert!(layer0.len() == 1);
        let layer1 = Camera::offset_lattice(&dx, &dy, 1);
        assert!(layer1.len() == 1);
        let layer2 = Camera::offset_lattice(&dx, &dy, 2);
        assert!(layer2.len() == 4);
        let layer3 = Camera::offset_lattice(&dx, &dy, 3);
        assert!(layer3.len() == 9);

        println!("Layer1:");
        for vec in layer1 {
            println!("({}, {})", vec.x(), vec.y());
        }

        println!("Layer2:");
        for vec in layer2 {
            println!("({}, {})", vec.x(), vec.y());
        }

        println!("Layer3:");
        for vec in layer3 {
            println!("({}, {})", vec.x(), vec.y());
        }
    }
}
