use std::fmt::Display;

pub mod database;

#[derive(Debug)]
pub enum Error {
    Common(common::error::Error),
    Database(database::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(error_value) => error_value.fmt(f),
            Error::Database(error_value) => error_value.fmt(f),
        }
    }
}

impl From<common::error::Error> for Error {
    fn from(value: common::error::Error) -> Self {
        Self::Common(value)
    }
}

impl From<common::error::user::Error> for Error {
    fn from(value: common::error::user::Error) -> Self {
        Self::Common(value.into())
    }
}

impl From<common::error::community::Error> for Error {
    fn from(value: common::error::community::Error) -> Self {
        Self::Common(value.into())
    }
}

impl From<common::error::message::Error> for Error {
    fn from(value: common::error::message::Error) -> Self {
        Self::Common(value.into())
    }
}

impl Into<common::error::Error> for Error {
    fn into(self) -> common::error::Error {
        match self {
            Error::Common(error) => error,
            Error::Database(error) => {
                eprintln!("Error: Database | {}", error);
                common::error::Error::Server
            }
        }
    }
}

impl From<database::Error> for Error {
    fn from(value: database::Error) -> Self {
        Self::Database(value)
    }
}

impl From<surrealdb::Error> for Error {
    fn from(value: surrealdb::Error) -> Self {
        database::Error::from(value).into()
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::Common(value.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Common(value.into())
    }
}

impl From<quinn::ConnectionError> for Error {
    fn from(value: quinn::ConnectionError) -> Self {
        Self::Common(common::error::network::Error::from(value).into())
    }
}

impl From<quinn::ConnectError> for Error {
    fn from(value: quinn::ConnectError) -> Self {
        Self::Common(value.into())
    }
}

impl From<quinn::ReadError> for Error {
    fn from(value: quinn::ReadError) -> Self {
        Self::Common(common::error::network::Error::from(value).into())
    }
}

impl From<quinn::ReadToEndError> for Error {
    fn from(value: quinn::ReadToEndError) -> Self {
        Self::Common(common::error::network::Error::from(value).into())
    }
}

impl From<quinn::WriteError> for Error {
    fn from(value: quinn::WriteError) -> Self {
        Self::Common(common::error::network::Error::from(value).into())
    }
}

impl From<quinn::ClosedStream> for Error {
    fn from(value: quinn::ClosedStream) -> Self {
        Self::Common(common::error::network::Error::from(value).into())
    }
}

impl From<bitcode::Error> for Error {
    fn from(value: bitcode::Error) -> Self {
        Self::Common(common::error::decode::Error::from(value).into())
    }
}
