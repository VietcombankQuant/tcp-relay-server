use std::{net::SocketAddr, str::FromStr};

use tokio::net::TcpListener;

use crate::{config::Config, handler};

pub(crate) async fn serve(config: Config) {
    let server_addr = match SocketAddr::from_str(&config.target_server) {
        Ok(addr) => addr,
        Err(err) => {
            log::error!(
                "Failed to resolve socket address {} with error {}",
                &config.relay_server,
                err.to_string()
            );
            return;
        }
    };
    log::info!("Target server is at address {}", server_addr);

    let local_addr = match SocketAddr::from_str(&config.relay_server) {
        Ok(addr) => addr,
        Err(err) => {
            log::error!(
                "Failed to resolve socket address {} with error {}",
                &config.relay_server,
                err.to_string()
            );
            return;
        }
    };

    let listener = match TcpListener::bind(local_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!(
                "Failed to listen at address {} with error {}",
                local_addr,
                err.to_string()
            );
            return;
        }
    };
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
