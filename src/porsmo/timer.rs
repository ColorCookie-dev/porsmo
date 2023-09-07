use crate::{prelude::*, CounterUIState, Alertable};
use crate::alert::{Alert, alert};
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
    pub alert: bool,
}

impl TimerState {
    pub fn new(start_time: Duration, target: Duration) -> Self {
        let counter = Counter::from(start_time).start();
        Self {
            counter,
            target,
            alert: false
        }
    }
}

impl CounterUIState for TimerState {
    fn show(&self, terminal: &mut TerminalHandler) -> Result<()> {
        let elapsed = self.counter.elapsed();
        if elapsed < self.target {
            let time_left = self.target.saturating_sub(elapsed);
            terminal
                .clear()?
                .info("Timer")?
                .set_foreground_color(running_color(self.counter.started()))?
                .print(fmt_time(time_left.as_secs()))?
                .info("[Q]: quit, [Space]: pause/resume")?
                .flush()?;
        } else {
            let excess_time = elapsed.saturating_sub(self.target);
            terminal
                .clear()?
                .info("Timer Has Ended")?
                .set_foreground_color(running_color(self.counter.started()))?
                .print(format_args!("+{}", fmt_time(excess_time.as_secs())))?
                .info("[Q]: quit, [Space]: pause/resume")?
                .flush()?;
        }
        Ok(())
    }

    fn handle_command(self, command: Command) -> Option<Self> {
        match command {
            Command::Quit => None,
            Command::Pause =>
                Some(Self { counter: self.counter.stop(), ..self}),
            Command::Resume =>
                Some(Self { counter: self.counter.start(), ..self}),
            Command::Toggle | Command::Enter =>
                Some(Self { counter: self.counter.toggle(), ..self}),
            _ => Some(self),
        }
    }
}

impl Alertable for TimerState {
    fn alerted(&self) -> bool {
        self.alert
    }

    fn set_alert(&mut self, alert: bool) {
        self.alert = alert;
    }

    fn should_alert(&self) -> bool {
        self.counter.elapsed() > self.target
    }

    fn alert(&mut self) {
        let title = "The timer has ended!";
        let message = format!(
            "Your Timer of {initial} has ended",
            initial = fmt_time(self.target.as_secs())
        );

        alert(title, message);
        self.alert = true;
    }
}
