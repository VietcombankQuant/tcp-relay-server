use std::{net::SocketAddr, str::FromStr};

use tokio::net::TcpListener;

mod handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let server_addr = SocketAddr::from_str("0.0.0.0:8088")?;
    log::info!("Target server is at address {}", server_addr);

    let local_addr = SocketAddr::from_str("0.0.0.0:8080")?;
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
