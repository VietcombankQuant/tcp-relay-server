use tokio::{net::TcpListener, sync::mpsc::UnboundedSender};

use crate::{book_keeper::ConnectionEvent, config::Config, handler};

pub(crate) async fn serve(config: Config, event_sender: UnboundedSender<ConnectionEvent>) {
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
                let handler = handler::handler(connection, config, event_sender.clone());
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
