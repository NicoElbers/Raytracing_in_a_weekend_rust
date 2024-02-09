use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::channel;
use std::sync::Arc;

use winit::event_loop::EventLoopProxy;

use crate::application::Events;
use crate::raytracing::color::Color;
use crate::raytracing::hittable::SceneObject;
use crate::raytracing::ray::Ray;
use crate::raytracing::render_pool::ThreadPool;
use crate::space::point3::Point3;
use crate::space::vec3::Vec3;
use crate::util::interval::Interval;
use crate::util::progress::{MessageType, ProgressBar};
use crate::util::random::XorShift;

#[derive(Debug, Clone, Copy)]
pub struct PixelRender {
    pub color: Color,
    pub x_loc: usize,
    pub y_loc: usize,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct CamData {
    pub focal_length: f64,
    pub viewport_height: f64,
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
#[allow(dead_code)]
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

// TODO: Refactor camera to use a builder, this is too many arguments :(
// TODO: Implement some sensible defaults for camera, again this is too much shit
#[derive(Debug, Clone)]
/// [Camera] stores information for a camera in a scene. It sets up a location
/// and some screen information (aspect ratio, height, focal length, etc).
#[allow(dead_code)]
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

    event_transmitter: EventLoopProxy<Events>,
}

impl Camera {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
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
        event_transmitter: EventLoopProxy<Events>,
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

        // NOTE: Calculate first pixel, this is the upper left of the viewport
        // Intentionally NOT the middle of the pixel
        let pixel00 = look_from - (focus_dist * w) - viewport_u / 2. - viewport_v / 2.;

        let defocus_radius = focus_dist * f64::tan(f64::to_radians(defocus_angle / 2.));
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

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
            event_transmitter,
        }
    }

    pub fn threaded_render(
        cam: &Arc<Self>,
        world: &Arc<SceneObject>,
        samples_sqrt: usize,
    ) -> Result<(), Box<dyn Error>> {
        // Log image to be rendered
        println!(
            r#"
            Multithreaded rendering
            Making an image of format:
                {} by {}
                {} samples
                {} max depth
            "#,
            cam.img.width,
            cam.img.height,
            samples_sqrt * samples_sqrt,
            cam.max_depth,
        );

        let offsets: Vec<Vec3> =
            Self::offset_lattice(&cam.pixel_delta_v, &cam.pixel_delta_u, samples_sqrt);
        let offsets: Arc<[Vec3]> = offsets.as_slice().into();

        let mut render_pool = ThreadPool::default();
        let mut rand = XorShift::default();

        let (pixel_transmitter, pixel_reciever) = channel::<PixelRender>();

        let pixel_transmitter = Arc::new(pixel_transmitter);

        assert!(render_pool.is_finished());

        #[allow(clippy::cast_precision_loss)]
        let mut progress_bar =
            ProgressBar::new(MessageType::Info, "Sending jobs", cam.height() as f64)?;

        for height in 0..cam.height() {
            progress_bar.update()?;
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

        let mut image_vec: Vec<Vec<Color>> = Vec::with_capacity(cam.height());

        #[allow(clippy::cast_precision_loss)]
        let mut progress_bar = ProgressBar::new(
            MessageType::Info,
            "Creating image vector",
            cam.height() as f64,
        )?;

        for height in 0..cam.height() {
            image_vec.push(Vec::<Color>::with_capacity(cam.width()));
            for _ in 0..cam.width() {
                image_vec[height].push(Color::black());
            }
            progress_bar.update()?;
        }

        #[allow(clippy::cast_precision_loss)]
        let mut progress_bar = ProgressBar::new(
            MessageType::Info,
            "Rendering pixels",
            (cam.width() * cam.height()) as f64,
        )?;

        while !render_pool.is_finished() {
            let pr = pixel_reciever.recv().expect("Couldn't get rendered pixel");
            image_vec[pr.y_loc][pr.x_loc] = pr.color;
            progress_bar.update()?;
            

            // TODO: make sure that exiting the program gracefully exits here
            cam.event_transmitter.send_event(Events::RenderPixel(pr))?;
        }

        println!("After pixel gathering");

        // Get file
        let file = File::create("img.ppm")?;
        let mut writer = BufWriter::new(&file);

        Color::wire_full_file(&mut image_vec, &mut writer)?;

        println!("Finished succesfully");
        Ok(())
    }

    fn ray_colors_lattice(
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

    fn ray_color(
        &self,
        r: Ray,
        world: &Arc<SceneObject>,
        rand: &mut XorShift,
        depth: usize,
    ) -> Color {
        if depth >= self.max_depth {
            return Color::black();
        }

        if let Some(record) = world.hit(&r, &Interval::from(0.01)) {
            if let Some((r, attenuation)) = record.mat().scatter(&r, &record, rand) {
                return attenuation * self.ray_color(r, world, rand, depth + 1);
            }

            return Color::black();
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

        // debug_assert!(
        //     ray_direction.len_squared() < 0.1,
        //     "Direction is too damn small"
        // );

        Ray::new(ray_origin, ray_direction)
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

    fn defocus_disk_sample(&self, rand: &mut XorShift) -> Point3 {
        let point = Vec3::random_vec_in_unit_disk(rand);

        self.cam.look_from + (point.x() * self.defocus_disk_u) + (point.y() * self.defocus_disk_v)
    }

    pub const fn width(&self) -> usize {
        self.img.width
    }

    pub const fn height(&self) -> usize {
        self.img.height
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
