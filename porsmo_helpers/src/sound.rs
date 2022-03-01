use rodio::{decoder::DecoderError, Decoder, OutputStream, PlayError, Sink, StreamError};
use std::io::Cursor;

#[derive(Debug)]
pub enum BellError {
    StreamError(StreamError),
    DecodeError(DecoderError),
    PlayError(PlayError),
}

impl std::fmt::Display for BellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::StreamError(error) => {
                write!(f, "Failed to notify: {:?}", error)
            }
            Self::DecodeError(error) => {
                write!(f, "Failed to notify: {:?}", error)
            }
            Self::PlayError(error) => {
                write!(f, "Failed to notify: {:?}", error)
            }
        }
    }
}

impl std::error::Error for BellError {}

impl From<PlayError> for BellError {
    fn from(error: rodio::PlayError) -> Self {
        Self::PlayError(error)
    }
}

impl From<DecoderError> for BellError {
    fn from(error: DecoderError) -> Self {
        Self::DecodeError(error)
    }
}

impl From<StreamError> for BellError {
    fn from(error: StreamError) -> Self {
        Self::StreamError(error)
    }
}

pub fn play_bell() -> Result<(), BellError> {
    let (_stream, stream_handle) = OutputStream::try_default()?;

    let audio = Decoder::new(Cursor::new(include_bytes!("notify_end.wav")))?;

    let sink = Sink::try_new(&stream_handle)?;
    sink.append(audio);
    sink.set_volume(0.1);
    sink.sleep_until_end();

    Ok(())
}
