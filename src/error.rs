use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum PorsmoError {
    #[error("Error entering raw mode in terminal")]
    FailedRawModeEnter(#[source] crossterm::ErrorKind),

    #[error("Error initializing terminal with alternate screen and mouse capture")]
    FailedInitialization(#[source] crossterm::ErrorKind),

    #[error("Wrong format for time")]
    WrongFormatError,

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    CrosstermError(#[from] crossterm::ErrorKind),
}
