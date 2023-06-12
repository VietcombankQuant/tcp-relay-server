use std::net::SocketAddr;

use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};

use crate::{
    book_keeper::{ConnectionEvent, ConnectionKey},
    config::Config,
};

pub(crate) async fn handler(
    connection: (TcpStream, SocketAddr),
    config: Config,
    event_sender: UnboundedSender<ConnectionEvent>,
) {
    // Log new connection with unique id
    let request_id = uuid::Uuid::new_v4().hyphenated().to_string();

    let (mut client_socket, client_addr) = connection;
    log::info!(
        "{} | Accept new connection from address {}",
        request_id,
        client_addr
    );

    // Connect to target server
    let mut server_socket = match TcpStream::connect(config.target_server).await {
        Ok(socket) => socket,
        Err(err) => {
            log::error!(
                "{} | Failed to connect to target address {} with error {}",
                request_id,
                config.target_server,
                err.to_string()
            );
            return;
        }
    };

    log::info!(
        "{} | Connected to target server {}",
        request_id,
        config.target_server
    );

    // Increase count for the connection
    let connection = ConnectionKey {
        client: client_addr.ip(),
        relay_server: config.relay_server,
        target_server: config.target_server,
    };
    _ = event_sender
        .send(ConnectionEvent::New(connection))
        .map_err(|err| {
            log::error!(
                "Failed to send message to unbounded channel with error {}",
                err.to_string()
            );
        });

    // Relay packets bidirectionaly
    let results = tokio::io::copy_bidirectional(&mut server_socket, &mut client_socket).await;
    match results {
        Ok(_) => {
            log::info!("{} | Disconnected from client {} ", request_id, client_addr);
            log::info!(
                "{} | Disconnected from target server {} ",
                request_id,
                config.target_server
            );
        }
        Err(err) => {
            log::error!(
                "{} | Failed to relay with error {}",
                request_id,
                err.to_string()
            );
        }
    };

    // Decrease count
    _ = event_sender
        .send(ConnectionEvent::Disconnect(connection))
        .map_err(|err| {
            log::error!(
                "Failed to send message to unbounded channel with error {}",
                err.to_string()
            );
        });
}
