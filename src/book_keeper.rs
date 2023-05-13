use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    ops::DerefMut,
    sync::Arc,
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
}

#[derive(Debug)]
pub(crate) struct ConnBookKeeper {
    receiver: UnboundedReceiver<ConnectionEvent>,
    counter: Arc<Mutex<HashMap<ConnectionKey, usize>>>,
}

impl ConnBookKeeper {
    pub(crate) fn new() -> (ConnBookKeeper, UnboundedSender<ConnectionEvent>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let counter = Arc::new(Mutex::new(HashMap::new()));
        (Self { receiver, counter }, sender)
    }

    pub(crate) async fn process_events(mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                ConnectionEvent::New(conn) => self.increase(conn).await,
                ConnectionEvent::Disconnect(conn) => self.decrease(conn).await,
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
}
