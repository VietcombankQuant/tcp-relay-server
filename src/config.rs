use std::{
    net::{AddrParseError, SocketAddr},
    path::Path,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    pub relay_server: SocketAddr,
    pub target_server: SocketAddr,
}

impl Config {
    pub(crate) async fn from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, ConfigError> {
        let configs = internal::Config::from_file(path).await?;
        let mut results = Vec::new();
        for config in configs {
            let result = Self {
                relay_server: parse_socket_addr(&config.relay_server)?,
                target_server: parse_socket_addr(&config.target_server)?,
            };
            results.push(result);
        }
        Ok(results)
    }
}

#[derive(Error, Debug)]
pub(crate) enum ConfigError {
    #[error("IO error {0}")]
    IO(std::io::Error),

    #[error("TOML decoding error {0}")]
    JsonDecode(serde_json::Error),

    #[error("socket address resolve error {0}")]
    SocketAddr(AddrParseError),
}

fn parse_socket_addr(addr: &str) -> Result<SocketAddr, ConfigError> {
    SocketAddr::from_str(addr).map_err(|err| ConfigError::SocketAddr(err))
}

mod internal {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct Config {
        #[serde(rename = "relay-server")]
        pub relay_server: String,

        #[serde(rename = "target-server")]
        pub target_server: String,
    }
}

impl internal::Config {
    pub(crate) async fn from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, ConfigError> {
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

        let config: Vec<Self> = match serde_json::from_str(&contents) {
            Ok(config) => config,
            Err(err) => return Err(ConfigError::JsonDecode(err)),
        };

        return Ok(config);
    }
}
