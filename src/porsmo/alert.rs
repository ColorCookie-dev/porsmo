use crate::prelude::*;
use rodio::{Decoder, OutputStream, Sink};
use std::{io::Cursor, thread};
use notify_rust::Notification;

pub fn notify_default(title: impl AsRef<str>, message: impl AsRef<str>) -> Result<()> {
    Notification::new()
        .appname("Porsmo")
        .summary(title.as_ref())
        .body(message.as_ref())
        .show()
        .with_context(|| "Failed to show notification")?;
    Ok(())
}
pub fn alert(title: impl Into<String>, message: impl Into<String>) {
    let title = title.into();
    let message = message.into();
    thread::spawn(move || {
        notify_default(title, message).unwrap();
        play_bell().unwrap();
    });
}

pub fn play_bell() -> Result<()> {
    let (_stream, stream_handle) =
        OutputStream::try_default().with_context(|| "failed to create an audio output stream")?;

    // let volume = 0.5;
    let audio = Decoder::new(Cursor::new(include_bytes!("notify_end.wav")))?;
    Sink::try_new(&stream_handle)
        .map(|sink| {
            sink.append(audio);
            // sink.set_volume(volume);
            sink.sleep_until_end();
        })
        .map_err(|_| anyhow!("failed to create a sink"))
}
