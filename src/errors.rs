use hyper::error::Error as HttpError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PocketError {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{1} (code {0})")]
    Proto(u16, String),
    #[error(transparent)]
    Io(#[from] IoError),
}