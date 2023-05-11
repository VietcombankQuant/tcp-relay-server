use std::{net::SocketAddr, str::FromStr};

use tokio::net::TcpListener;

mod config;
mod handler;
mod utils;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::setup_logger();

    let config = Config::from_file("config.toml").await?;

    let server_addr = SocketAddr::from_str(&config.target_server)?;
    log::info!("Target server is at address {}", server_addr);

    let local_addr = SocketAddr::from_str(&config.relay_server)?;
    let listener = TcpListener::bind(local_addr).await?;
    log::info!("TCP relay server is listening at address {}", local_addr);

    loop {
        match listener.accept().await {
            Ok((client_socket, client_addr)) => {
                let handler = handler::handler(client_socket, client_addr, server_addr);
                tokio::spawn(handler);
            }
            Err(err) => {
                log::error!(
                    "Failed to accept connection from client with error {}",
                    err.to_string()
                );
            }
        };
    }
}
