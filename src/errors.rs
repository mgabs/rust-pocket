use std::error::Error;
use std::io::Error as IoError;
use hyper::error::{Error as HttpError};

#[derive(Debug)]
pub enum PocketError {
    Http(HttpError),
    Json(serde_json::Error),
    Proto(u16, String),
    Io(IoError),
}


impl From<serde_json::Error> for PocketError {
    fn from(err: serde_json::Error) -> PocketError {
        PocketError::Json(err)
    }
}

impl From<IoError> for PocketError {
    fn from(err: IoError) -> PocketError {
        PocketError::Io(err)
    }
}

impl From<HttpError> for PocketError {
    fn from(err: HttpError) -> PocketError {
        PocketError::Http(err)
    }
}

impl Error for PocketError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            PocketError::Http(ref e) => Some(e),
            PocketError::Json(ref e) => Some(e),
            PocketError::Proto(..) => None,
            PocketError::Io(ref e) => Some(e),
        }
    }
}

impl std::fmt::Display for PocketError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            PocketError::Http(ref e) => e.fmt(fmt),
            PocketError::Json(ref e) => e.fmt(fmt),
            PocketError::Proto(ref code, ref msg) => {
                fmt.write_str(&*format!("{} (code {})", msg, code))
            }
            PocketError::Io(ref e) => e.fmt(fmt),
        }
    }
}