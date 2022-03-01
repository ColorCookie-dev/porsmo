use crate::{
    input::{listen_command, Command},
    terminal::RawTerm,
};
use anyhow::Result;
use porsmo::stopwatch::Stopwatch;
use porsmo_helpers::fmt_time;
use std::{thread, time::Duration};
use termion::color;

pub fn stopwatch(time: u64) -> Result<()> {
    let mut stdout = RawTerm::default();
    let rx = listen_command();
    let mut counter = Stopwatch::new(Duration::from_secs(time));

    loop {
        stdout.clear()?;

        stdout.set_color(color::Magenta)?;
        stdout.write_line("Stopwatch")?;

        if counter.is_running() {
            stdout.set_color(color::Green)?;
        } else {
            stdout.set_color(color::Red)?;
        }

        stdout.write_line(fmt_time(counter.counter_at().as_secs()))?;

        stdout.set_color(color::LightYellow)?;
        stdout.write_line("[Q]: Quit, [Space]: pause/resume")?;

        stdout.flush()?;

        match rx.try_recv() {
            Ok(Command::Quit) => {
                break;
            }

            Ok(Command::Space) | Ok(Command::Enter) => {
                counter.toggle();
            }

            _ => (),
        }

        thread::sleep(Duration::from_millis(100));
    }

    stdout.destroy();
    println!("+{}", fmt_time(counter.counter_at().as_secs()));

    Ok(())
}
