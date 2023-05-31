use crate::{
    alert::alert,
    format::fmt_time,
    input::{listen_for_inputs, Command},
    stopwatch::default_stopwatch_loop,
    terminal::TerminalHandler,
};
use anyhow::Result;
use porsmo::{counter::Counter, timer::Timer};
use std::{sync::mpsc::Receiver, thread, time::Duration};

pub fn timer(time: u64) -> Result<Duration> {
    let mut c = Timer::new(Duration::from_secs(time));
    let mut terminal = &mut TerminalHandler::new()?;
    let rx = listen_for_inputs();
    let counter_ended_at;

    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                c.end_count();
                counter_ended_at = c.elapsed();
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
            alert("Timer ended!".into(), "You Porsmo timer has ended".into());
            counter_ended_at = start_excess_counting(&mut terminal, &rx)?;
            break;
        }

        terminal.show_counter(
            "Timer",
            fmt_time(c.elapsed()),
            c.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(counter_ended_at)
}

fn start_excess_counting(terminal: &mut TerminalHandler, rx: &Receiver<Command>)
    -> Result<Duration> {
    default_stopwatch_loop(rx, 0, move |st| {
        terminal.show_counter(
            "Timer has ended",
            format!("+{}", fmt_time(st.elapsed())),
            st.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )?;

        Ok(())
    })
}
