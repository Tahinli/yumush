use std::fmt::Display;

use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Error {
    Inner(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Inner(error_value) => error_value.fmt(f),
        }
    }
}

impl From<bitcode::Error> for Error {
    fn from(value: bitcode::Error) -> Self {
        Self::Inner(value.to_string())
    }
}
