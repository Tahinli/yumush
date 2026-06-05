use std::fmt::Display;

use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Error {
    Connection(String),
    Read(String),
    ReadToEnd(String),
    Write(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Connection(error_value) => error_value.fmt(f),
            Error::Read(error_value) => error_value.fmt(f),
            Error::ReadToEnd(error_value) => error_value.fmt(f),
            Error::Write(error_value) => error_value.fmt(f),
        }
    }
}

impl From<quinn::ConnectionError> for Error {
    fn from(value: quinn::ConnectionError) -> Self {
        Self::Connection(value.to_string())
    }
}

impl From<quinn::ReadError> for Error {
    fn from(value: quinn::ReadError) -> Self {
        Self::Read(value.to_string())
    }
}

impl From<quinn::ReadToEndError> for Error {
    fn from(value: quinn::ReadToEndError) -> Self {
        Self::Read(value.to_string())
    }
}

impl From<quinn::WriteError> for Error {
    fn from(value: quinn::WriteError) -> Self {
        Self::Write(value.to_string())
    }
}
