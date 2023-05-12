use std::{io::ErrorKind, net::SocketAddr};

use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

use crate::config::Config;

pub(crate) async fn handler(connection: (TcpStream, SocketAddr), config: Config) {
    // Log new connection with unique id
    let request_id = uuid::Uuid::new_v4().hyphenated().to_string();

    let (client_socket, client_addr) = connection;
    log::info!(
        "{} | Accept new connection from address {}",
        request_id,
        client_addr
    );

    // Connect to target server
    let server_socket = match TcpStream::connect(config.target_server).await {
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

    let (client_reader, client_writer) = client_socket.into_split();
    let (server_reader, server_writer) = server_socket.into_split();
    let results = futures::try_join!(
        relay(client_reader, server_writer),
        relay(server_reader, client_writer)
    );

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
}

async fn relay(reader: OwnedReadHalf, writer: OwnedWriteHalf) -> anyhow::Result<()> {
    const BUF_SIZE: usize = 256;
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        reader.readable().await?;

        match reader.try_read(&mut buffer) {
            Ok(n) => match n {
                0 => {
                    return Ok(());
                }
                n => {
                    writer.writable().await?;
                    writer.try_write(&buffer[..n])?;
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }
}
