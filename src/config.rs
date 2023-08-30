use std::{fs, io};

use crate::messages::MessageMagicNumber;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Config {
    pub dest_addr: String,
    pub network_type: MessageMagicNumber,
}

impl Config {
    pub fn load_config(file_name: &str) -> Result<Self, ConfigLoadError> {
        let str_config = fs::read_to_string(file_name)?;
        let config: Config = serde_json::from_str(&str_config)?;
        Ok(config)
    }
}

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("Error while reading file: {0}")]
    Read(
        #[from]
        #[source]
        io::Error,
    ),

    #[error("Error while deserializing config: {0}")]
    Serde(
        #[from]
        #[source]
        serde_json::Error,
    ),
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_config_loadable() {
        let config = Config::load_config("config.json").unwrap();
        assert_eq!(config.network_type, MessageMagicNumber::Main);
    }
}
