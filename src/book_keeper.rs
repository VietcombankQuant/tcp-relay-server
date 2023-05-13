use std::{
    collections::HashMap,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct ConnectionKey {
    pub client: IpAddr,
    pub relay_server: SocketAddr,
    pub target_server: SocketAddr,
}

pub(crate) enum ConnectionEvent {
    New(ConnectionKey),
    Disconnect(ConnectionKey),
    ListAll,
}

#[derive(Debug)]
pub(crate) struct ConnBookKeeper {
    sender: UnboundedSender<ConnectionEvent>,
    receiver: UnboundedReceiver<ConnectionEvent>,
    counter: Arc<Mutex<HashMap<ConnectionKey, usize>>>,
}

impl ConnBookKeeper {
    pub(crate) fn new() -> (ConnBookKeeper, UnboundedSender<ConnectionEvent>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let counter = Arc::new(Mutex::new(HashMap::new()));

        (
            Self {
                sender: sender.clone(),
                receiver,
                counter,
            },
            sender,
        )
    }

    pub(crate) async fn process_events(mut self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                _ = sender.send(ConnectionEvent::ListAll).map_err(|err| {
                    log::error!(
                        "Failed to send message to unbounded channel with error {}",
                        err.to_string()
                    );
                });
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });

        while let Some(event) = self.receiver.recv().await {
            match event {
                ConnectionEvent::New(conn) => self.increase(conn).await,
                ConnectionEvent::Disconnect(conn) => self.decrease(conn).await,
                ConnectionEvent::ListAll => self.list_all().await,
            }
        }
    }

    async fn increase(&self, connection: ConnectionKey) {
        let mut guard = self.counter.lock().await;
        let count = guard.deref_mut().entry(connection).or_default();
        *count += 1;
    }

    async fn decrease(&self, connection: ConnectionKey) {
        let mut guard = self.counter.lock().await;
        let count = guard
            .entry(connection)
            .and_modify(|count| *count -= 1)
            .or_default();

        if *count == 0 {
            guard.remove_entry(&connection);
        }
    }

    async fn list_all(&self) {
        let guard = self.counter.lock().await;
        let connections = guard
            .deref()
            .iter()
            .map(|(conn, count)| (conn.clone(), count.clone()))
            .collect::<Vec<_>>();

        tokio::spawn(async move {
            use prettytable::{row, Table};

            let mut table = Table::new();
            _ = table.add_row(row![
                "Relay Server",
                "Target Server",
                "Client IP",
                "Number Connections"
            ]);

            for (conn, count) in connections.iter() {
                let row = row![conn.relay_server, conn.target_server, conn.client, count];
                _ = table.add_row(row);
            }

            if connections.is_empty() {
                _ = table.add_empty_row();
            }

            let records = table.to_string();
            log::info!("Active connections:\n{}", records);
        });
    }
}
