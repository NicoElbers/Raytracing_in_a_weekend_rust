// TODO: Factor all the raytrace code out to a module
// TODO: Implement a winit application to show progress while we're rendering
// TODO: Render to png instead of ppm
// TODO: Make this motherfucker a beast by implementing WebGPU rendering

mod application;
mod raytracing;
mod space;
mod util;

use std::{error::Error, thread};

use application::Application;

fn main() -> Result<(), Box<dyn Error>> {
    // raytracing::simple()?;
    // raytracing::complex()?;
    // raytracing::super_simple()?;
    // raytracing::threads()?;

    let width = 1000;
    let height = 1000;

    let (mut application, event_loop) = Application::create(width, height)?;

    let event_loop_proxy = event_loop.create_proxy();

    thread::scope(|scope| {
        scope.spawn(move || {
            if let Err(err) = raytracing::complex(width, height, event_loop_proxy) {
                eprintln!();
                eprintln!("Render thread errored with {err}");
            }
        });

        if let Err(err) = application.run(event_loop) {
            eprintln!();
            eprintln!("Application errored with error {err}");
        }
    });

    Ok(())
}
