use std::error::Error;
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::sync::mpsc::channel;
use std::sync::Arc;

use crate::raytracing::color::Color;
use crate::raytracing::hittable::SceneObject;
use crate::raytracing::ray::Ray;
use crate::raytracing::render_pool::RenderPool;
use crate::space::point3::Point3;
use crate::space::vec3::Vec3;
use crate::util::interval::Interval;
use crate::util::random::XorShift;

// TODO: Refactor camera to use a builder, this is too many arguments :(
// TODO: Implement some sensible defaults for camera, again this is too much shit
#[derive(Default, Debug, Clone, Copy)]
/// [Camera] stores information for a camera in a scene. It sets up a location
/// and some screen information (aspect ratio, height, focal length, etc).
pub struct Camera {
    img: ImgData,
    cam: CamData,
    basis: BasisVecs,
    // NOTE: pixel00 is equal to the top left of the viewport. Since a lattice
    // of offsets is used, they are offset from the top left of a pixel
    pixel00: Point3,
    max_depth: usize,

    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,

    defocus_angle: f64,
    focus_dist: f64,

    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    #[must_use]
    pub fn new(
        img_height: usize,
        img_width: usize,
        max_depth: usize,
        focal_length: f64,
        fov: f64,
        look_from: Point3,
        look_to: Point3,
        vup: Vec3,
        defocus_angle: f64,
        focus_dist: f64,
    ) -> Self {
        // Viewport
        let theta = f64::to_radians(fov);
        let h = f64::tan(theta / 2.);
        let viewport_height: f64 = 2. * h * focus_dist;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let viewport_width: f64 = viewport_height * (img_width as f64 / img_height as f64);

        let w: Vec3 = (look_from - look_to).into();
        let w = w.unit();

        let u = Vec3::cross(&vup, w).unit();

        let v = Vec3::cross(&w, u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let pixel_delta_u = viewport_u / img_width as f64;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let pixel_delta_v = viewport_v / img_height as f64;

        // Calculate first pixel, this is the upper left of the viewport
        // Intentionally NOT the middle of the pixel
        let pixel00 = look_from - (focus_dist * w) - viewport_u / 2. - viewport_v / 2.;

        let defocus_radius = focus_dist * f64::tan(f64::to_radians(defocus_angle / 2.));
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        // Data helper structs
        let cam = CamData::new(
            focal_length,
            viewport_height,
            viewport_width,
            fov,
            look_from,
            look_to,
            vup,
        );

        let img = ImgData::new(img_height, img_width);

        let basis = BasisVecs::new(u, v, w);

        Self {
            img,
            cam,
            basis,
            pixel00,
            max_depth,
            pixel_delta_u,
            pixel_delta_v,
            defocus_angle,
            focus_dist,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    fn update_prog(&self, counter: usize, last_prog: &mut usize) -> std::io::Result<()> {
        let prog = counter / (self.img.height * self.img.width / 100);

        if *last_prog != prog {
            let mut out = stdout();
            out.write_all(b"\r\x1b[2K")?;
            print!("[ INFO ] {prog}% done");
            out.flush()?;
            *last_prog = prog;
        }

        Ok(())
    }

    /// Render the world as displayed by the world parameter. It will render all
    /// objects
    ///
    /// # Errors
    ///
    /// This function will return an error if it cannot open a file called img.ppm
    /// or if it can later not write to that file.
    pub fn render(&self, world: &Arc<SceneObject>, samples_sqrt: usize) -> std::io::Result<()> {
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

        let mut last_prog: usize = 0;
        let mut out = stdout();
        // out.write_all(b"\r\x1b[2K")?;
        print!("[ INFO ] 0% done");
        out.flush()?;

        for height in 0..self.img.height {
            self.update_prog(height, &mut last_prog)?;
            for width in 0..self.img.width {
                // let ray = self.get_ray(i, j, &mut rand);

                // let avg_color = Self::ray_colors_random(&ray, world, &mut random, samples_sqrt, &self.pixel_delta_u, &self.pixel_delta_v);
                let avg_color =
                    Self::ray_colors_lattice(self, width, height, world, &offsets, &mut rand);

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

    pub fn threaded_render(
        cam: &Arc<Self>,
        world: &Arc<SceneObject>,
        samples_sqrt: usize,
    ) -> Result<(), Box<dyn Error>> {
        #[derive(Debug, Clone, Copy)]
        struct PixelRender {
            pub color: Color,
            pub x_loc: usize,
            pub y_loc: usize,
        }

        // Log image to be rendered
        println!(
            "Multithreaded rendering\nMaking an image of format:\n\t{} by {}\n\t{} samples\n\t{} max depth",
            cam.img.width,
            cam.img.height,
            samples_sqrt * samples_sqrt,
            cam.max_depth,
        );

        let offsets: Vec<Vec3> =
            Self::offset_lattice(&cam.pixel_delta_v, &cam.pixel_delta_u, samples_sqrt);
        let offsets: Arc<[Vec3]> = offsets.as_slice().into();

        let mut render_pool = RenderPool::default();
        let mut rand = XorShift::default();

        let (pixel_transmitter, pixel_reciever) = channel::<PixelRender>();

        let pixel_transmitter = Arc::new(pixel_transmitter);

        assert!(render_pool.is_finished());

        for height in 0..cam.height() {
            for width in 0..cam.width() {
                let mut rand = rand.copy_reset();
                let camera = cam.clone();
                let world = world.clone();
                let offsets = offsets.clone();
                let event_transmitter = pixel_transmitter.clone();

                render_pool.send_job(Box::new(move || {
                    let color =
                        camera.ray_colors_lattice(width, height, &world, &offsets, &mut rand);

                    event_transmitter
                        .send(PixelRender {
                            color,
                            x_loc: width,
                            y_loc: height,
                        })
                        .expect("Couldn't send pixel back");
                }))?;
            }
        }

        assert!(!render_pool.is_finished());

        println!("Done sending jobs");

        let mut image_vec: Vec<Vec<Color>> = Vec::with_capacity(cam.height());

        for height in 0..cam.height() {
            image_vec.push(Vec::<Color>::with_capacity(cam.width()));
            for _ in 0..cam.width() {
                image_vec[height].push(Color::default());
            }
        }

        println!("Done setup vector");
        println!("Now listening for messages");

        // TODO: Factor out updating progress *more*
        let mut last_prog: usize = 0;
        let mut out = stdout();
        // out.write_all(b"\r\x1b[2K")?;
        print!("[ INFO ] 0% done");
        out.flush()?;

        let mut counter = 0;
        while !render_pool.is_finished() {
            cam.update_prog(counter, &mut last_prog)?;

            let pr = pixel_reciever.recv().expect("Couldn't get rendered pixel");

            // dbg!(pr);

            image_vec[pr.y_loc][pr.x_loc] = pr.color;

            counter += 1;
        }

        let mut out = stdout();
        out.write_all(b"\r\x1b[2K")?;
        println!("[ INFO ]     done!");
        println!("Now writing file");
        out.flush()?;

        // Get file
        let file = File::create("img.ppm")?;
        let mut writer = BufWriter::new(&file);

        Color::wire_full_file(&mut image_vec, &mut writer)?;

        println!("Finished succesfully");
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn ray_colors_lattice(
        &self,
        width: usize,
        height: usize,
        world: &Arc<SceneObject>,
        offsets: &[Vec3],
        rand: &mut XorShift,
    ) -> Color {
        debug_assert!(!offsets.is_empty());

        #[allow(clippy::cast_precision_loss)]
        let avg_color = offsets
            .iter()
            .map(|offset| {
                let r = self.get_ray(width, height, offset, rand);
                self.ray_color(r, world, rand, 0)
            })
            .fold(Color::default(), |acc, clr| acc + clr)
            / offsets.len() as f64;
        avg_color
    }

    #[allow(dead_code)]
    fn ray_colors_random(
        &self,
        ray: Ray,
        world: &Arc<SceneObject>,
        rand: &mut XorShift,
        samples: usize,
        dx: &Vec3,
        dy: &Vec3,
    ) -> Color {
        let mut total_color = Color::default();
        for _ in 0..samples {
            let ray = ray.offset_dir(&(*dx * rand.next_01() + *dy * rand.next_01()));
            let color = Self::ray_color(self, ray, world, rand, 0);
            total_color = total_color + color;
        }

        #[allow(clippy::cast_precision_loss)]
        let avg_color = total_color / samples as f64;

        avg_color
    }

    fn ray_color(
        &self,
        r: Ray,
        world: &Arc<SceneObject>,
        rand: &mut XorShift,
        depth: usize,
    ) -> Color {
        if depth >= self.max_depth {
            return Color::new(0., 0., 0.);
        }

        if let Some(record) = world.hit(&r, &Interval::from(0.01)) {
            if let Some((r, attenuation)) = record.mat().scatter(&r, &record, rand) {
                return attenuation * self.ray_color(r, world, rand, depth + 1);
            }

            return Color::default();
        }

        let unit_dir = r.dir().unit();
        let a = 0.5 * (unit_dir.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    fn get_ray(&self, i: usize, j: usize, offset: &Vec3, rand: &mut XorShift) -> Ray {
        let offset: Point3 = Into::into(*offset);

        let pixel_loc = self.pixel00 + (i * self.pixel_delta_u) + (j * self.pixel_delta_v);
        let pixel_sample: Point3 = pixel_loc + offset;

        let ray_origin = if self.defocus_angle <= 0. {
            self.cam.look_from
        } else {
            self.defocus_disk_sample(rand)
        };

        let ray_direction: Vec3 = (pixel_sample - ray_origin).into();

        if ray_direction.len_squared() < 0.1 {
            println!("FUUCK ME WHY DIRECTION");
            // dbg!(ray_direction);
        }

        Ray::new(ray_origin, ray_direction)
    }

    pub(crate) fn offset_lattice(dx: &Vec3, dy: &Vec3, num_layers: usize) -> Vec<Vec3> {
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

    pub const fn width(&self) -> usize {
        self.img.width
    }

    pub const fn height(&self) -> usize {
        self.img.height
    }

    fn defocus_disk_sample(&self, rand: &mut XorShift) -> Point3 {
        let point = Vec3::random_vec_in_unit_disk(rand);

        self.cam.look_from + (point.x() * self.defocus_disk_u) + (point.y() * self.defocus_disk_v)
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct ImgData {
    pub height: usize,
    pub width: usize,
}

impl ImgData {
    const fn new(img_height: usize, img_width: usize) -> Self {
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
    pub look_from: Point3,
    pub look_to: Point3,
    pub vup: Vec3,
}

impl CamData {
    const fn new(
        focal_length: f64,
        viewport_height: f64,
        viewport_width: f64,
        fov: f64,
        look_from: Point3,
        look_to: Point3,
        vup: Vec3,
    ) -> Self {
        Self {
            focal_length,
            viewport_height,
            viewport_width,
            fov,
            look_from,
            look_to,
            vup,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct BasisVecs {
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

impl BasisVecs {
    const fn new(u: Vec3, v: Vec3, w: Vec3) -> Self {
        Self { u, v, w }
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