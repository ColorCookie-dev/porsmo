#[derive(Debug, thiserror::Error)]
pub enum PorsmoError {
    #[error("IO Error: {0}")]
    FlushError(#[source] std::io::Error),

    #[error("Error entering raw mode in terminal")]
    FailedRawModeEnter(#[source] crossterm::ErrorKind),

    #[error("Error initializing terminal with alternate screen and mouse capture")]
    FailedInitialization(#[source] crossterm::ErrorKind),

    #[error("Error clearing terminal")]
    FailedClear(#[source] crossterm::ErrorKind),

    #[error("Failed to set foreground color")]
    ForegroundColorSetFailed(#[source] crossterm::ErrorKind),

    #[error("Failed to print to screen")]
    FailedPrint(#[source] crossterm::ErrorKind),

    #[error(transparent)]
    CrosstermError(#[from] crossterm::ErrorKind),
}

