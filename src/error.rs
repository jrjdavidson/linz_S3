use std::error;
use std::fmt::Write;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum MyError {
    #[error("Neither spatial filter nor content filter were provided. This will cause a search of the whole bucket- aborting.")]
    NoFilterProvided,
    #[error("Cannot specify both dimensions (height_m, width_m) and a coordinate range. Please choose one.")]
    DimensionAndCoordinateRange,
    #[error("STAC error: {0}")]
    StacError(#[from] stac::Error),
    #[error("STAC error: {0}")]
    SendError(#[from] Box<tokio::sync::mpsc::error::SendError<stac::Item>>),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

impl MyError {
    /// Recursively formats the error and its sources into a readable string.
    pub fn report(&self) {
        let mut err: &dyn error::Error = self;
        let mut s = format!("{}", err);
        while let Some(src) = err.source() {
            let _ = write!(s, "\nCaused by: {}", src);
            err = src;
        }
        ::log::error!("Error: {}", s);
    }
}
