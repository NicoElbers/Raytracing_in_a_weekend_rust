// TODO: Render to png instead of ppm
// TODO: Make this motherfucker a beast by implementing WebGPU rendering

mod application;
mod raytracing;
mod space;
mod util;

use std::{env::args, error::Error, process::exit, thread};

use application::Application;

struct Config {
    pub height: usize,
    pub width: usize,
    pub sample_sqrt: usize,
    pub preview: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            height: 1080,
            width: 1920,
            sample_sqrt: 10,
            preview: false,
        }
    }
}

impl Config {
    fn parse_args() -> Config {
        let mut config = Config::default();
        let args: Vec<_> = args().collect();
        for (idx, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--height" | "-h" => {
                    if let Some(Ok(height)) = args //
                        .get(idx + 1)
                        .map(|s| s.parse::<usize>())
                    {
                        config.height = height
                    } else {
                        eprintln!("Usage: --height <number>");
                        exit(1)
                    }
                }
                "--width" | "-w" => {
                    if let Some(Ok(width)) = args //
                        .get(idx + 1)
                        .map(|s| s.parse::<usize>())
                    {
                        config.width = width
                    } else {
                        eprintln!("Usage: --width <number>");
                        exit(1)
                    }
                }
                "--samplesqrt" | "-s" => {
                    if let Some(Ok(sample_sqrt)) = args //
                        .get(idx + 1)
                        .map(|s| s.parse::<usize>())
                    {
                        config.sample_sqrt = sample_sqrt
                    } else {
                        eprintln!("Usage: --samplesqrt <number>");
                        exit(1)
                    }
                }
                "--preview" | "-p" => {
                    config.preview = true;
                }
                "--help" => {
                    println!("Use the application like this:");
                    println!("\t-h --height\t:\tSet the height of the image");
                    println!("\t--width -w\t:\tSet the width of the image");
                    println!(
                        "\t--samplesqrt -s\t:\tSet the sqrt of the samples used for the image"
                    );
                    println!("\t--preview -p\t:\tSet whether a preview window is displayed");
                    exit(0);
                }
                _ => (),
            }
        }
        config
    }

    fn build_application(&self) -> Result<Option<Application>, Box<dyn Error>> {
        if self.preview {
            Ok(Some(Application::create(self.width, self.height)?))
        } else {
            Ok(None)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse_args();

    let application = config.build_application()?;
    let proxy = application
        .as_ref()
        .map(|application| application.create_proxy());

    thread::scope(|scope| {
        scope.spawn(move || {
            if let Err(err) = raytracing::complex(&config, proxy) {
                eprintln!();
                eprintln!("Render thread errored with {err}");
            }
        });

        if let Some(application) = application {
            if let Err(err) = application.run() {
                eprintln!();
                eprintln!("Application errored with error {err}");
            }
        }
    });

    Ok(())
}
