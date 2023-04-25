use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    #[serde(rename = "relay-server")]
    pub relay_server: String,

    #[serde(rename = "target-server")]
    pub target_server: String,
}

#[derive(Error, Debug)]
pub(crate) enum ConfigError {
    #[error("IO error {0}")]
    IO(std::io::Error),

    #[error("TOML decoding error {0}")]
    TomlDecode(toml::de::Error),
}

impl Config {
    pub(crate) async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let mut file = match File::open(path).await {
            Ok(file) => file,
            Err(err) => return Err(ConfigError::IO(err)),
        };

        let mut contents = String::default();
        match file.read_to_string(&mut contents).await {
            Ok(_) => {}
            Err(err) => return Err(ConfigError::IO(err)),
        };

        let config: Config = match toml::from_str(&contents) {
            Ok(config) => config,
            Err(err) => return Err(ConfigError::TomlDecode(err)),
        };

        return Ok(config);
    }
}
