use notify_rust::Notification;
use rodio::{Decoder, OutputStream, Sink};
use std::{io::Cursor, thread};

#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("Failed to show notification")]
    FailedToNotify(#[from] notify_rust::error::Error),

    #[error(transparent)]
    SoundError(#[from] SoundError),
}

pub fn notify_default(title: impl AsRef<str>, message: impl AsRef<str>) -> Result<(), AlertError> {
    Notification::new()
        .appname("Porsmo")
        .summary(title.as_ref())
        .body(message.as_ref())
        .show()?;
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

#[derive(Debug, Clone, Copy)]
pub struct Alerter(bool);

impl Default for Alerter {
    fn default() -> Self {
        Self(false)
    }
}

impl Alerter {
    pub fn alert_once(&mut self, title: impl Into<String>, message: impl Into<String>) {
        if !self.0 {
            self.0 = true;
            alert(title, message);
        }
    }

    pub fn reset(&mut self) {
        self.0 = false;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SoundError {
    #[error(transparent)]
    StreamError(#[from] rodio::StreamError),

    #[error(transparent)]
    DevicesError(#[from] rodio::DevicesError),

    #[error(transparent)]
    DecoderError(#[from] rodio::decoder::DecoderError),

    #[error("No devices found")]
    NoDevice,
}

impl From<rodio::PlayError> for SoundError {
    fn from(err: rodio::PlayError) -> Self {
        match err {
            rodio::PlayError::NoDevice => Self::NoDevice,
            rodio::PlayError::DecoderError(e) => Self::DecoderError(e),
        }
    }
}

pub fn play_bell() -> Result<(), SoundError> {
    let (_stream, stream_handle) = OutputStream::try_default()?;

    // let volume = 0.5;
    let audio = Decoder::new(Cursor::new(include_bytes!("notify_end.wav")))?;
    Sink::try_new(&stream_handle).map(|sink| {
        sink.append(audio);
        // sink.set_volume(volume);
        sink.sleep_until_end();
    })?;

    Ok(())
}
