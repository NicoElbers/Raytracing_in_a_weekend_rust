use std::error::Error;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    platform::x11::{WindowBuilderExtX11, XWindowType},
    window::{Window, WindowBuilder},
};

use crate::raytracing::camera::PixelRender;

#[derive(Clone, Copy, Debug)]
pub enum Events {
    RenderPixel(PixelRender),
}

pub struct Application {
    pixels: Pixels,
    window: Window,
    width: usize,
    height: usize,
}

impl Application {
    pub fn new(width: usize, height: usize, pixels: Pixels, window: Window) -> Self {
        Self {
            pixels,
            window,
            width,
            height,
        }
    }

    pub fn create(
        width: usize,
        height: usize,
    ) -> Result<(Self, EventLoop<Events>), Box<dyn Error>> {
        let event_loop = EventLoopBuilder::<Events>::with_user_event().build()?;

        let application_width = u32::try_from(width).expect("fuck you");
        let application_height = u32::try_from(height).expect("fuck you");

        let window = {
            let size = LogicalSize::new(application_width, application_height);
            WindowBuilder::new()
                .with_title("Ray tracer")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_max_inner_size(size)
                .with_x11_window_type(vec![XWindowType::Utility])
                .build(&event_loop)?
        };

        let pixels = {
            let surface_texture =
                SurfaceTexture::new(application_width, application_height, &window);
            Pixels::new(application_width, application_height, surface_texture)?
        };

        Ok((Self::new(width, height, pixels, window), event_loop))
    }

    pub fn run(
        &mut self,
        event_loop: EventLoop<Events>,
    ) -> Result<(), winit::error::EventLoopError> {
        event_loop.set_control_flow(ControlFlow::Wait);

        // println!("{:?}", self.pixels);

        event_loop.run(move |event, target: &EventLoopWindowTarget<Events>| {
            match event {
                Event::UserEvent(event) => {
                    Self::handle_user_event(&mut self.pixels, event, &self.window, self.width);
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    let _ = self.pixels.resize_surface(new_size.width, new_size.height);
                    let _ = self.pixels.resize_buffer(
                        u32::try_from(self.width).expect("fuck you"),
                        u32::try_from(self.height).expect("fuck you"),
                    );

                    self.window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // TODO: Maybe add timeout in here?
                    Self::render(&self.pixels);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    target.exit();
                }
                _ => (),
                // Event::NewEvents(_) => todo!(),
                // Event::WindowEvent { window_id, event } => todo!(),
                // Event::DeviceEvent { device_id, event } => todo!(),
                // Event::Suspended => todo!(),
                // Event::Resumed => todo!(),
                // Event::AboutToWait => todo!(),
                // Event::LoopExiting => todo!(),
                // Event::MemoryWarning => unimplemented!(),
            }
        })
    }

    fn draw(width: usize, pixels: &mut Pixels, render: PixelRender) {
        // for pixel in pixels.frame_mut().chunks_exact_mut(4) {
        //     pixel[0] = 0xAF;
        //     pixel[1] = 0xAF;
        //     pixel[2] = 0xAF;
        //     pixel[3] = 0xFF;
        // }

        let x = render.x_loc;
        let y = render.y_loc;

        let pos = y * width + x;
        let pos = pos * 4;

        let color = render.color;
        let color = color * 255.;

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let red = color.r() as u8;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let green = color.g() as u8;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let blue = color.b() as u8;

        let frame = pixels.frame_mut();
        frame[pos] = red;
        frame[pos + 1] = green;
        frame[pos + 2] = blue;
        frame[pos + 3] = 0xFF;
    }

    fn render(pixels: &Pixels) {
        if let Err(err) = pixels.render() {
            eprintln!("Pixel render errored with {err}");
        }
    }

    fn handle_user_event(pixel: &mut Pixels, event: Events, window: &Window, width: usize) {
        match event {
            Events::RenderPixel(pixel_render) => {
                Self::draw(width, pixel, pixel_render);
                window.request_redraw();
            }
        }
    }
}
