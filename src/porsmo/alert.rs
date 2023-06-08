use anyhow::{Context, Result, anyhow};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use crate::notification::notify_default;
use std::thread;

pub fn alert(title: String, message: String) {
    thread::spawn(move || {
        notify_default(&title, &message).unwrap();
        play_bell().unwrap();
    });
}

pub fn play_bell() -> Result<()> {
    let (_stream, stream_handle) =
        OutputStream::try_default().with_context(|| "failed to create an audio output stream")?;

    let audio = Decoder::new(Cursor::new(include_bytes!("notify_end.wav")))?;
    Sink::try_new(&stream_handle)
        .map(|sink| {
            sink.append(audio);
            sink.set_volume(0.1);
            sink.sleep_until_end();
        })
        .map_err(|_| anyhow!("failed to create a sink"))
}
