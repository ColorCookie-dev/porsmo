use crate::counter::Counter;
use crate::input::{listen_for_inputs, Command};
use crate::terminal::{show_view, TermRawMode};
use anyhow::Result;
use std::{sync::mpsc::Receiver, time::Instant};

pub struct Timer {
    started: Instant,
    counter: u64,
    status: Status,
    stdout_raw: TermRawMode,
    input_receiver: Receiver<Command>,
}

enum Status {
    Running,
    Paused,
    Ended,
}

impl Timer {
    pub fn new(count: u64) -> Self {
        let stdout_raw = TermRawMode::new();
        let input_receiver = listen_for_inputs();

        Self {
            started: Instant::now(),
            counter: count,
            status: Status::Running,
            input_receiver,
            stdout_raw,
        }
    }
}

impl Counter for Timer {
    fn has_ended(&self) -> bool {
        matches!(self.status, Status::Ended)
    }

    fn is_running(&self) -> bool {
        matches!(self.status, Status::Running)
    }

    fn is_paused(&self) -> bool {
        matches!(self.status, Status::Paused)
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.counter += self.started.elapsed().as_secs();
            self.status = Status::Paused;
        }
    }

    fn resume(&mut self) {
        if self.is_paused() {
            self.status = Status::Running;
            self.started = Instant::now();
        }
    }

    fn counter(&self) -> u64 {
        if self.is_running() {
            self.counter + self.started.elapsed().as_secs()
        } else {
            self.counter
        }
    }

    fn update(&mut self) -> Result<()> {
        match self.input_receiver.try_recv() {
            Ok(Command::Quit) => {
                self.status = Status::Ended;
                return Ok(());
            }

            Ok(Command::Pause) => {
                self.pause();
            }

            Ok(Command::Resume) => {
                self.resume();
            }

            Ok(Command::Toggle) | Ok(Command::Enter) => {
                self.toggle();
            }

            _ => (),
        }

        let running = self.is_running();
        let counter = self.counter();
        show_view(&mut self.stdout_raw.stdout, counter, running)?;

        Ok(())
    }
}
