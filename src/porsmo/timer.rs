use crate::{
    alert::alert,
    format::fmt_time,
    input::Command,
    stopwatch::default_stopwatch_loop,
    terminal::TerminalHandler,
};
use crate::prelude::*;
use crossterm::event;
use porsmo::{counter::Counter, timer::Timer};
use std::{thread, time::Duration};

pub struct TimerUI;

impl TimerUI {
    pub fn new(time: Duration) -> Result<Duration> {
        timer(time)
    }

    pub fn from_secs(time: u64) -> Result<Duration> {
        timer(Duration::from_secs(time))
    }
}

pub fn timer(time: Duration) -> Result<Duration> {
    let mut c = Timer::new(time);
    let mut terminal = &mut TerminalHandler::new()?;
    let counter_ended_at;

    loop {
        if event::poll(Duration::from_millis(250))
            .with_context(|| "Polling failed")? {
            let event = event::read().with_context(|| "Failed to read event")?;
            let command = Command::from(event);
            match command {
                Command::Quit => {
                    c.end_count();
                    counter_ended_at = c.elapsed();
                    break;
                },
                Command::Pause => c.pause(),
                Command::Resume => c.resume(),
                Command::Toggle | Command::Enter => c.toggle(),
                _ => (),
            }
        }

        if c.has_ended() {
            c.end_count();
            alert("Timer ended!".into(), "You Porsmo timer has ended".into());
            counter_ended_at = start_excess_counting(&mut terminal)?;
            break;
        }

        terminal.show_counter(
            "Timer",
            fmt_time(c.elapsed()),
            c.is_running(),
            "[Q]: quit, [Space]: pause/resume",
            "",
        )?;

    }

    Ok(counter_ended_at)
}

fn start_excess_counting(terminal: &mut TerminalHandler)
    -> Result<Duration> {
    default_stopwatch_loop(Duration::ZERO, move |st| {
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
