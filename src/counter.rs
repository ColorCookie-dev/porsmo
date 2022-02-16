use crate::input::{listen_for_inputs, Command};
use crate::terminal::{show_view, TermRawMode};
use anyhow::Result;
use std::{thread, time::Duration};

pub trait Counter {
    fn is_running(&self) -> bool;

    fn pause(&mut self);

    fn resume(&mut self);

    fn toggle(&mut self) {
        if self.is_running() {
            self.pause();
        } else {
            self.resume();
        }
    }

    fn counter(&self) -> u64;

    fn check_end(&self) -> bool;

    fn count(&mut self) -> Result<u64> {
        let mut stdout = &mut TermRawMode::new().stdout;
        let receiver = listen_for_inputs();

        while !self.check_end() {
            match receiver.try_recv() {
                Ok(Command::Quit) => {
                    break;
                }

                Ok(Command::Pause) => {
                    self.pause();
                }

                Ok(Command::Resume) => {
                    self.resume();
                }

                Ok(Command::Toggle) => {
                    self.toggle();
                }

                _ => (),
            }

            show_view(&mut stdout, self.counter(), self.is_running())?;
            thread::sleep(Duration::from_millis(500));
        }

        Ok(self.counter())
    }
}
