use std::fs;

use serde::Deserialize;

use crate::messages::MessageMagicNumber;

#[derive(Deserialize)]
pub struct Config {
    pub dest_addr: String,
    pub network_type: MessageMagicNumber,
}

impl Config {
    pub fn load_config() -> Result<Self, ()> {
        let str_config = fs::read_to_string("config.json").unwrap();
        let config: Config = serde_json::from_str(&str_config).unwrap();
        Ok(config)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_config_loadable() {
        let config = Config::load_config().unwrap();
        assert_eq!(config.network_type, MessageMagicNumber::Main);
    }
}
