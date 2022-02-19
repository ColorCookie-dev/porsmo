use crate::{
    input::{listen_for_inputs, Command},
    terminal::{show_view, TermRawMode},
};
use anyhow::Result;
use porsmo::{counter::Counter, stopwatch::Stopwatch};
use std::{io::Write, sync::mpsc::Receiver, thread, time::Duration};

pub fn default_stopwatch_loop<T>(
    time: u64,
    stdout: &mut T,
    rx: &Receiver<Command>,
    update: impl Fn(&mut T, &Stopwatch) -> Result<()>,
) -> Result<u64>
where
    T: Write,
{
    let mut st = Stopwatch::new(time);

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

        update(stdout, &st)?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(st.counter())
}

pub fn stopwatch(time: u64) -> Result<u64> {
    let mut stdout = &mut TermRawMode::new().stdout;
    let rx = listen_for_inputs();

    default_stopwatch_loop(time, &mut stdout, &rx, move |stdout, st| {
        show_view(stdout, st.counter(), st.is_running())
    })
}
