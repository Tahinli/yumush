use std::fmt::Display;

use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Error {
    AddressParse(String),
    InputOutput(String),
    Connect(String),
    Connection(String),
    Read(String),
    ReadToEnd(String),
    ReadExact(String),
    Write(String),
    ClosedStream(String),
    ConnectionError(String),
    ChannelClosed,
    ReadBoundExceed(usize, usize),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AddressParse(error_value) => error_value.fmt(f),
            Error::InputOutput(error_value) => error_value.fmt(f),
            Error::Connect(error_value) => error_value.fmt(f),
            Error::Connection(error_value) => error_value.fmt(f),
            Error::Read(error_value) => error_value.fmt(f),
            Error::ReadToEnd(error_value) => error_value.fmt(f),
            Error::ReadExact(error_value) => error_value.fmt(f),
            Error::Write(error_value) => error_value.fmt(f),
            Error::ClosedStream(error_value) => error_value.fmt(f),
            Error::ConnectionError(error_value) => error_value.fmt(f),
            Error::ChannelClosed => write!(f, "Channel Closed"),
            Error::ReadBoundExceed(actual, expected) => write!(
                f,
                "Read Bound Exceed | Actual = {} | Expected = {}",
                actual, expected
            ),
        }
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::AddressParse(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::InputOutput(value.to_string())
    }
}

impl From<quinn::ConnectionError> for Error {
    fn from(value: quinn::ConnectionError) -> Self {
        Self::Connection(value.to_string())
    }
}

impl From<quinn::ConnectError> for Error {
    fn from(value: quinn::ConnectError) -> Self {
        Self::AddressParse(value.to_string())
    }
}

impl From<quinn::ReadError> for Error {
    fn from(value: quinn::ReadError) -> Self {
        Self::Read(value.to_string())
    }
}

impl From<quinn::ReadToEndError> for Error {
    fn from(value: quinn::ReadToEndError) -> Self {
        Self::ReadToEnd(value.to_string())
    }
}

impl From<quinn::ReadExactError> for Error {
    fn from(value: quinn::ReadExactError) -> Self {
        Self::ReadExact(value.to_string())
    }
}

impl From<quinn::WriteError> for Error {
    fn from(value: quinn::WriteError) -> Self {
        Self::Write(value.to_string())
    }
}

impl From<quinn::ClosedStream> for Error {
    fn from(value: quinn::ClosedStream) -> Self {
        Self::ClosedStream(value.to_string())
    }
}
