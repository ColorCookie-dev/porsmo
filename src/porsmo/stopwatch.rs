use crate::{
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use crate::prelude::*;
use crossterm::event;
use porsmo::{counter::Countable, stopwatch::Stopwatch};
use std::{thread, time::Duration};

pub struct StopwatchUI;

impl StopwatchUI {
    pub fn new(time: Duration) -> Result<Duration> {
        stopwatch(time)
    }

    pub fn from_secs(time: u64) -> Result<Duration> {
        stopwatch(Duration::from_secs(time))
    }
}

pub fn default_stopwatch_loop(
    time: Duration,
    mut update: impl FnMut(&Stopwatch) -> Result<()>,
) -> Result<Duration> {
    let mut st = Stopwatch::new(time);

    loop {
        if event::poll(Duration::from_millis(250))
            .with_context(|| "Polling failed")? {
            let event = event::read().with_context(|| "Failed to read event")?;
            let command = Command::from(event);
            match command {
                Command::Quit => {
                    st.end_count();
                    break;
                }
                Command::Pause => st.pause(),
                Command::Resume => st.resume(),
                Command::Toggle | Command::Enter => st.toggle(),
                _ => (),
            }
        }

        update(&st)?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(st.elapsed())
}

pub fn stopwatch(time: Duration) -> Result<Duration> {
    let mut terminal = TerminalHandler::new()?;

    default_stopwatch_loop(time, move |st| {
        terminal.show_counter(
            "StopWatch",
            fmt_time(st.elapsed()),
            st.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )
    })
}
