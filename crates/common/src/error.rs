use std::fmt::Display;

use bitcode::{Decode, Encode};

pub mod community;
pub mod decode;
pub mod file_operation;
pub mod message;
pub mod network;
pub mod user;

#[derive(Debug, Clone, Encode, Decode)]
pub enum Error {
    Authenticate,
    User(user::Error),
    Message(message::Error),
    Community(community::Error),
    FileOperation(file_operation::Error),
    Network(network::Error),
    Decode(decode::Error),
    Server,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Authenticate => write!(f, "Authenticate"),
            Error::User(error_value) => error_value.fmt(f),
            Error::Message(error_value) => error_value.fmt(f),
            Error::Community(error_value) => error_value.fmt(f),
            Error::FileOperation(error_value) => error_value.fmt(f),
            Error::Network(error_value) => error_value.fmt(f),
            Error::Decode(error_value) => error_value.fmt(f),
            Error::Server => write!(f, "Server"),
        }
    }
}

impl From<user::Error> for Error {
    fn from(value: user::Error) -> Self {
        Error::User(value)
    }
}

impl From<message::Error> for Error {
    fn from(value: message::Error) -> Self {
        Error::Message(value)
    }
}

impl From<community::Error> for Error {
    fn from(value: community::Error) -> Self {
        Error::Community(value)
    }
}

impl From<file_operation::Error> for Error {
    fn from(value: file_operation::Error) -> Self {
        Error::FileOperation(value)
    }
}

impl From<network::Error> for Error {
    fn from(value: network::Error) -> Self {
        Error::Network(value)
    }
}

impl From<decode::Error> for Error {
    fn from(value: decode::Error) -> Self {
        Error::Decode(value)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::Network(value.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Network(value.into())
    }
}

impl From<quinn::ConnectionError> for Error {
    fn from(value: quinn::ConnectionError) -> Self {
        Self::Network(value.into())
    }
}

impl From<quinn::ConnectError> for Error {
    fn from(value: quinn::ConnectError) -> Self {
        Self::Network(value.into())
    }
}

impl From<quinn::ReadError> for Error {
    fn from(value: quinn::ReadError) -> Self {
        Self::Network(value.into())
    }
}

impl From<quinn::ReadToEndError> for Error {
    fn from(value: quinn::ReadToEndError) -> Self {
        Self::Network(value.into())
    }
}

impl From<quinn::WriteError> for Error {
    fn from(value: quinn::WriteError) -> Self {
        Self::Network(value.into())
    }
}

impl From<quinn::ClosedStream> for Error {
    fn from(value: quinn::ClosedStream) -> Self {
        Self::Network(value.into())
    }
}

impl From<bitcode::Error> for Error {
    fn from(value: bitcode::Error) -> Self {
        Self::Decode(value.into())
    }
}
