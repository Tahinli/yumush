use std::fmt::Display;

use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Error {
    FileOpen(String),
    Read,
    Empty,
    Split,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileOpen(error_value) => write!(f, "File open is failed| {}", error_value),
            Error::Read => write!(f, "Read file context is failed"),
            Error::Empty => write!(f, "File is empty"),
            Error::Split => write!(f, "Splitting context is failed"),
        }
    }
}
