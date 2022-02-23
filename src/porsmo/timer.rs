use crate::{
    format::fmt_time,
    input::{listen_for_inputs, Command},
    stopwatch::default_stopwatch_loop,
    terminal::{show_counter, TermRawMode},
};
use anyhow::Result;
use porsmo::{counter::Counter, timer::Timer};
use std::{io::Write, sync::mpsc::Receiver, thread, time::Duration};

pub fn timer(time: u64) -> Result<u64> {
    use ui::*;

    let mut c = Timer::new(time);
    let mut stdout = &mut TermRawMode::new().stdout;
    let rx = listen_for_inputs();
    let counter_ended_at;

    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                c.end_count();
                counter_ended_at = c.counter();
                break;
            }

            Ok(Command::Pause) => {
                c.pause();
            }

            Ok(Command::Resume) => {
                c.resume();
            }

            Ok(Command::Toggle) | Ok(Command::Enter) => {
                c.toggle();
            }

            _ => (),
        }

        if c.has_ended() {
            c.end_count();
            alert_timer_end();
            counter_ended_at = start_excess_counting(stdout, &rx)?;
            break;
        }

        show_counter(
            &mut stdout,
            "Timer",
            fmt_time(c.counter()),
            c.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(counter_ended_at)
}

fn start_excess_counting(stdout: &mut impl Write, rx: &Receiver<Command>) -> Result<u64> {
    default_stopwatch_loop(stdout, rx, 0, move |stdout, st| {
        show_counter(
            stdout,
            "Timer has ended",
            format!("+{}", fmt_time(st.counter())),
            st.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )?;

        Ok(())
    })
}

mod ui {
    use crate::{notification::notify_default, sound::play_bell};
    use std::thread;

    pub fn alert_timer_end() {
        thread::spawn(move || {
            notify_default("Timer ended!", "You Porsmo timer has ended").unwrap();
            play_bell().unwrap();
        });
    }
}
