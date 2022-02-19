use crate::counter::Counter;
use crate::format::fmt_time;
use crate::input::{listen_for_inputs, Command};
use crate::notification::notify_default;
use crate::sound::play_bell;
use crate::terminal::{
    clear, show_message, show_message_green, show_message_red, show_view, TermRawMode,
};
use anyhow::Result;
use std::{io::Stdout, sync::mpsc::Receiver, thread, time::Instant};
use termion::raw::RawTerminal;

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
    TimerEnd,       // When Timer expires
    TimerEndPaused, // When Timer expires
    Ended,          // Program End
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

    fn alert(&self) {
        thread::spawn(move || {
            notify_default("You break ended!", "Time for some work").unwrap();
            play_bell().unwrap();
        });
    }

    fn end_timer(&mut self) {
        self.pause();
        self.status = Status::TimerEnd;
    }

    fn counter_now(&self) -> u64 {
        let elapsed = self.started.elapsed().as_secs();
        if self.counter > elapsed {
            self.counter - elapsed
        } else {
            0
        }
    }

    fn get_mut_stdout(&mut self) -> &mut RawTerminal<Stdout> {
        &mut self.stdout_raw.stdout
    }

    fn show_extended_time(&mut self) -> Result<()> {
        let counter = self.counter();
        match self.status {
            Status::TimerEnd => {
                show_message_green(self.get_mut_stdout(), &format!("-{}", fmt_time(counter)), 2)?
            }
            Status::TimerEndPaused => {
                show_message_red(self.get_mut_stdout(), &format!("-{}", fmt_time(counter)), 2)?
            }
            _ => (),
        };
        Ok(())
    }

    fn show_timer_end(&mut self) -> Result<()> {
        clear(self.get_mut_stdout())?;
        show_message_red(self.get_mut_stdout(), "Timer has ended", 0)?;
        show_message(
            self.get_mut_stdout(),
            "[Q]: Quit, [Space]: Toggle excess counter",
            1,
        )?;
        self.show_extended_time()?;

        Ok(())
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

    fn counter(&self) -> u64 {
        match self.status {
            Status::Running => self.counter_now(),
            Status::TimerEnd => self.counter + self.started.elapsed().as_secs(),
            _ => self.counter,
        }
    }

    fn pause(&mut self) {
        match self.status {
            Status::Running => {
                self.counter = self.counter_now();
                self.status = Status::Paused;
            }
            Status::TimerEnd => {
                self.counter = self.counter();
                self.status = Status::TimerEndPaused;
            }
            _ => (),
        }
    }

    fn resume(&mut self) {
        match self.status {
            Status::Paused => {
                self.status = Status::Running;
                self.started = Instant::now();
            }
            Status::TimerEndPaused => {
                self.status = Status::TimerEnd;
                self.started = Instant::now();
            }
            _ => (),
        }
    }

    fn toggle(&mut self) {
        match self.status {
            Status::Paused | Status::TimerEndPaused => {
                self.resume();
            }
            Status::Running | Status::TimerEnd => {
                self.pause();
            }
            _ => (),
        }
    }

    fn end_count(&mut self) {
        self.pause();
        self.status = Status::Ended;
    }

    fn update(&mut self) -> Result<()> {
        match self.input_receiver.try_recv() {
            Ok(Command::Quit) => {
                self.end_count();
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

        if self.is_running() && self.counter_now() == 0 {
            self.end_timer();
            self.alert();
            return Ok(());
        }

        match self.status {
            Status::Running | Status::Paused => {
                let running = self.is_running();
                let counter = self.counter();
                show_view(&mut self.stdout_raw.stdout, counter, running)?;
            }
            Status::TimerEnd | Status::TimerEndPaused => {
                self.show_timer_end()?;
            }
            _ => (),
        }
        Ok(())
    }
}
