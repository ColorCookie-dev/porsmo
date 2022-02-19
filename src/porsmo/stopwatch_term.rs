use crate::{
    input::{listen_for_inputs, Command},
    terminal::{show_view, TermRawMode},
};
use anyhow::Result;
use porsmo::{counter::Counter, stopwatch::Stopwatch};
use std::{thread, time::Duration};

pub fn stopwatch(time: u64) -> Result<u64> {
    let mut st = Stopwatch::new(time);
    let mut stdout = &mut TermRawMode::new().stdout;
    let rx = listen_for_inputs();

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

        show_view(&mut stdout, st.counter(), st.is_running())?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(st.counter())
}
