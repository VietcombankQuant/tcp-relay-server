use tokio::net::TcpListener;

use crate::{config::Config, handler};

pub(crate) async fn serve(config: Config) {
    log::info!("Target server is at address {}", config.target_server);

    let listener = match TcpListener::bind(config.relay_server).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!(
                "Failed to listen at address {} with error {}",
                config.relay_server,
                err.to_string()
            );
            return;
        }
    };

    log::info!(
        "TCP relay server is listening at address {}",
        config.relay_server
    );

    loop {
        match listener.accept().await {
            Ok((client_socket, client_addr)) => {
                let handler = handler::handler(client_socket, client_addr, config.target_server);
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
