use crate::{
    format::fmt_time,
    input::{listen_for_inputs, Command},
    terminal::TerminalHandler,
};
use anyhow::Result;
use porsmo::{counter::Counter, stopwatch::Stopwatch};
use std::{sync::mpsc::Receiver, thread, time::Duration};

pub fn default_stopwatch_loop(
    rx: &Receiver<Command>,
    time: u64,
    mut update: impl FnMut(&Stopwatch) -> Result<()>,
) -> Result<Duration> {
    let mut st = Stopwatch::new(Duration::from_secs(time));

    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                st.end_count();
                break;
            }

            Ok(Command::Pause) => {
                st.pause();
            }

            Ok(Command::Resume) => {
                st.resume();
            }

            Ok(Command::Toggle) | Ok(Command::Enter) => {
                st.toggle();
            }

            _ => (),
        }

        update(&st)?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(st.elapsed())
}

pub fn stopwatch(time: u64) -> Result<Duration> {
    let mut terminal = TerminalHandler::new()?;
    let rx = listen_for_inputs();

    default_stopwatch_loop(&rx, time, move |st| {
        terminal.show_counter(
            "StopWatch",
            fmt_time(st.elapsed()),
            st.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )
    })
}
