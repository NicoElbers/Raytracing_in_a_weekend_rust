use std::{
    io::{self, stdout, Write},
    time::{Duration, Instant},
};

use terminal_size::{terminal_size, Width};

#[allow(dead_code)]
pub enum MessageType {
    Info,
    Error,
}

impl MessageType {
    const fn get_message(&self) -> &str {
        match self {
            Self::Info => "[ INFO ]",
            Self::Error => "[ ERROR ]",
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ProgressBar<'a> {
    // goal: usize,
    step_count: f64,
    update_at_count: f64,
    percent: usize,
    message: &'a str,
    message_type: MessageType,
    start_time: Instant,
}

impl<'a> ProgressBar<'a> {
    pub fn new(message_type: MessageType, message: &'a str, goal: f64) -> io::Result<Self> {
        let update_at_count: f64 = (goal - 1.) / 100.;
        let last_update_time = Instant::now();

        let progress_bar = Self {
            // goal,
            step_count: 0.,
            update_at_count,
            percent: 0,
            message,
            message_type,
            start_time: last_update_time,
        };

        progress_bar.print_message()?;

        Ok(progress_bar)
    }

    fn print_message(&self) -> io::Result<()> {
        let terminal_size = terminal_size();
        let percent = self.percent;

        let elapsed = self.start_time.elapsed();

        // println!("{elapsed:?}");

        let eta = if percent >= 100 {
            Duration::ZERO
        } else {
            let elapsed_per_percent = elapsed
                / (percent + 1)
                    .try_into()
                    .expect("Cannot get time elapsed per percent");

            elapsed_per_percent
                * (100 - percent)
                    .try_into()
                    .expect("Cannot unwraped time remaining")
        };

        let eta = eta.as_millis();

        let milis = eta % 1000;
        let seconds = (eta / 1_000) % 60;
        let minutes = (eta / 60_000) % 60;
        let hours = eta / 3_600_000;

        let eta = format!("{hours:02}:{minutes:02}:{seconds:02}:{milis:03}");

        let left_side = format!("{} {} ", self.message_type.get_message(), self.message);
        let right_side = format!(" {percent}% done | ETA: {eta}");

        let padding_amount = match terminal_size {
            Some((Width(w), _)) => w as usize - left_side.len() - right_side.len(),
            None => 1,
        };

        let padding = "·".repeat(padding_amount);

        let line = format!("{left_side}{padding}{right_side}");

        let mut out = stdout();
        // Clear line
        out.write_all(b"\r\x1b[2K")?;

        print!("{line}");

        if self.percent >= 100 {
            println!();
        }

        // Flush stdout
        out.flush()?;

        Ok(())
    }

    pub fn update_by(&mut self, by: f64) -> io::Result<()> {
        self.step_count += by;

        if self.step_count >= self.update_at_count {
            self.step_count -= self.update_at_count;
            self.percent += 1;

            if self.step_count >= self.update_at_count {
                self.update_by(0.)?;
            } else {
                self.print_message()?;
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> io::Result<()> {
        self.update_by(1.)
    }
}
