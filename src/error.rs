use std::{io, net::AddrParseError};

use thiserror::Error;

use crate::config::ConfigLoadError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config load error: {0}")]
    ConfigLoad(
        #[from]
        #[source]
        ConfigLoadError,
    ),

    #[error("error parsing address: {0}")]
    AddressParse(
        #[from]
        #[source]
        AddrParseError,
    ),

    #[error("IO error: {0}")]
    Io(
        #[from]
        #[source]
        io::Error,
    ),

    #[error("parse message error: {0}")]
    ParseMessage(
        #[from]
        #[source]
        Box<bincode::ErrorKind>,
    ),

    #[error("unexpected error: {0}")]
    Unexpected(
        #[source]
        #[from]
        anyhow::Error,
    ),
}
