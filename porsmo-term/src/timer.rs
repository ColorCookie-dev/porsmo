use crate::{
    input::{listen_command, Command},
    terminal::RawTerm,
};
use anyhow::Result;
use porsmo::timer::{CountType, Timer};
use porsmo_helpers::{alert, fmt_time};
use std::{thread, time::Duration};
use termion::color;

pub fn timer(time: u64) -> Result<()> {
    let mut stdout = RawTerm::default();
    let rx = listen_command();
    let mut alerted = false;
    let mut counter = Timer::new(Duration::from_secs(time));

    loop {
        stdout.clear()?;
        match counter.counter_at() {
            CountType::Count(c) => {
                stdout.set_color(color::Magenta)?;
                stdout.write_line("Timer")?;

                if counter.is_running() {
                    stdout.set_color(color::Green)?;
                } else {
                    stdout.set_color(color::Red)?;
                }

                stdout.write_line(fmt_time(c.as_secs()))?;
            }
            CountType::Exceed(c) => {
                stdout.set_color(color::Magenta)?;
                stdout.write_line("Timer ended!")?;

                if counter.is_running() {
                    stdout.set_color(color::Green)?;
                } else {
                    stdout.set_color(color::Red)?;
                }

                stdout.write_line(format!("+{}", fmt_time(c.as_secs())))?;
            }
        }

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

        if counter.has_ended() && alerted == false {
            alerted = true;
            alert("Timer ended".into(), "Your timer has ended".into());
        }

        thread::sleep(Duration::from_millis(100));
    }

    stdout.destroy();
    match counter.counter_at() {
        CountType::Count(c) => println!("{}", fmt_time(c.as_secs())),
        CountType::Exceed(c) => println!("+{}", fmt_time(c.as_secs())),
    };

    Ok(())
}
