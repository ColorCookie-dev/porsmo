use crate::{
    input::{listen_command, Command},
    writeraw,
};
use anyhow::Result;
use porsmo::{counter::*, stopwatch::Stopwatch};
use porsmo_helpers::fmt_time;
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};
use termion::{color, raw::IntoRawMode};

pub fn stopwatch(time: u64) -> Result<()> {
    let mut counter = Stopwatch::new(Duration::from_secs(time));

    {
        let mut stdout = stdout().into_raw_mode()?;
        let rx = listen_command();

        loop {
            writeraw! {
                stdout, clear,
                %text "Stopwatch", color color::Magenta, (1, 1)%,
                %text fmt_time(counter.counter_at().as_secs()),
                    runcolor counter.is_running(), (1, 2)%,
                %text "[Q]: Quit, [Space]: pause/resume", color color::LightYellow, (1, 3)%
            }

            stdout.flush()?;

            match rx.try_recv() {
                Ok(Command::Quit) => break,
                Ok(Command::Space) | Ok(Command::Enter) => counter.toggle(),
                _ => (),
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    println!();
    // println!("+{}", fmt_time(counter.counter_at().as_secs()));

    Ok(())
}
