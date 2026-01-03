use crate::prelude::*;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::io::Cursor;
use mac_notification_sys::*;

pub fn alert(title: &str, message: &str) -> Result<()> {
    // notifica::notify("Porsmo", "Your Timer is Up!").unwrap();
    play_bell().unwrap();
    Ok(())
}

// pub fn notify_default(title: impl AsRef<str>, message: impl AsRef<str>) -> Result<()> {
//     notify_rust::Notification::new()
//         .appname("Porsmo")
//         .summary("Porsmo Update")
//         .summary("Your Timer is Up!")
//         .timeout(6000)
//         .show()?;
//     Ok(())
// }

pub fn play_bell() -> Result<()> {
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    // let volume = 0.5;
    let sink = Sink::connect_new(&stream_handle.mixer());
    let source = Decoder::new(Cursor::new(include_bytes!("notify_end.wav")))?;
    stream_handle.mixer().add(source);
    sink.sleep_until_end();
    Ok(())
}
