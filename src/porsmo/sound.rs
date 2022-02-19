use anyhow::{bail, Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

pub fn play_bell() -> Result<()> {
    let (_stream, stream_handle) =
        OutputStream::try_default().with_context(|| "failed to create an audio output stream")?;

    let audio = Decoder::new(Cursor::new(include_bytes!("notify_end.wav")))?;
    match Sink::try_new(&stream_handle) {
        Ok(sink) => {
            sink.append(audio);
            sink.set_volume(0.1);
            sink.sleep_until_end();
            Ok(())
        }
        Err(_) => bail!("failed to create a sink"),
    }
}
