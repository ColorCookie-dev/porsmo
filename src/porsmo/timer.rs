use crate::input::{TIMEOUT, get_event};
use crate::prelude::*;
use crate::alert::Alert;
use crate::terminal::running_color;
use crate::{
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use porsmo::counter::Counter;
use std::time::Duration;

#[derive(Debug)]
pub struct TimerState {
    pub counter: Counter,
    pub target: Duration,
}

impl TimerState {
    pub fn run(terminal: &mut TerminalHandler, target: Duration) -> Result<()> {
        let counter = Counter::from(Duration::ZERO).start();
        let mut state = Self { counter, target };

        loop {
            state.show(terminal)?;
            if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
                match state.handle_command(cmd) {
                    Some(new_state) => state = new_state,
                    None => return Ok(()),
                }
            }
        }
    }

    pub fn handle_command(self, command: Command) -> Option<Self> {
        match command {
            Command::Quit => None,
            Command::Pause =>
                Some(Self { counter: self.counter.stop(), ..self}),
            Command::Resume =>
                Some(Self { counter: self.counter.start(), ..self}),
            Command::Toggle | Command::Enter =>
                Some(Self { counter: self.counter.start(), ..self}),
            _ => Some(self),
        }
    }

    pub fn show(&self, terminal: &mut TerminalHandler) -> Result<()> {
        let elapsed = self.counter.elapsed();
        if elapsed < self.target {
            let time_left = self.target.saturating_sub(elapsed);
            terminal
                .clear()?
                .info("Timer")?
                .set_foreground_color(running_color(self.counter.started()))?
                .print(fmt_time(time_left))?
                .info("[Q]: quit, [Space]: pause/resume")?
                .flush()?;
        } else {
            let excess_time = elapsed.saturating_sub(self.target);
            let title = "The timer has ended!";
            let message = format!(
                "Your Timer of {initial} has ended",
                initial = fmt_time(self.target)
            );

            terminal
                .clear()?
                .info("Timer Has Ended")?
                .set_foreground_color(running_color(self.counter.started()))?
                .print(format_args!("+{}", fmt_time(excess_time)))?
                .info("[Q]: quit, [Space]: pause/resume")?
                .flush()?;
        }
        Ok(())
    }
}

