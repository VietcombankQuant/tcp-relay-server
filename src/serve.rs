use tokio::net::TcpListener;

use crate::{config::Config, handler, book_keeper::ConnectionRegistry};

pub(crate) async fn serve(config: Config, registry: ConnectionRegistry) {
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
        "Server is listening at \"{}\" and forward packets to \"{}\"",
        config.relay_server,
        config.target_server
    );

    loop {
        match listener.accept().await {
            Ok(connection) => {
                let registry = registry.clone();
                let handler = handler::handler(connection, config, registry);
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
