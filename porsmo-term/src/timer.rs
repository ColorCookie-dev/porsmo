use crate::{
    input::{listen_command, Command},
    program_tick_duration, writeraw,
};
use anyhow::Result;
use porsmo::{counter::*, timer::*};
use porsmo_helpers::{alert, fmt_time};
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};

use termion::{color, raw::IntoRawMode};

pub fn timer(time: Duration) -> Result<()> {
    let mut counter = Timer::new(time);

    {
        let mut stdout = stdout().into_raw_mode()?;
        let rx = listen_command();
        let mut alerted = false;

        loop {
            writeraw!(stdout, clear);
            match counter.checked_counter_at() {
                CountType::Count(c) => {
                    writeraw! {
                        stdout,
                        %text "Timer", color color::Magenta, (1, 1)%,
                        %text fmt_time(c), runcolor counter.is_running(), (1, 2)%,
                    }
                }
                CountType::Exceed(c) => {
                    writeraw! {
                        stdout,
                        %text "Timer ended!", color color::Magenta, (1, 1)%,
                        %text format_args!("+{}", fmt_time(c)),
                            runcolor counter.is_running(), (1, 2)%,
                    }
                }
            }

            writeraw!(
                stdout,
                %text "[Q]: Quit, [Space]: pause/resume", color color::LightYellow, (1, 3)%
            );

            stdout.flush()?;

            match rx.try_recv() {
                Ok(Command::Quit) => break,
                Ok(Command::Space) | Ok(Command::Enter) => counter.toggle(),
                _ => (),
            }

            if counter.has_ended() && alerted == false {
                alerted = true;
                alert("Timer ended".into(), "Your timer has ended".into());
            }

            thread::sleep(program_tick_duration!());
        }
    }

    println!();
    Ok(())
}
