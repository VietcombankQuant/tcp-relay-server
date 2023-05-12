use std::{
    collections::HashMap,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::Mutex;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct ConnectionKey {
    pub client: SocketAddr,
    pub relay_server: SocketAddr,
    pub target_server: SocketAddr,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ConnectionRegistry(Arc<Mutex<HashMap<ConnectionKey, usize>>>);

impl ConnectionRegistry {
    pub(crate) async fn increase(&self, connection: ConnectionKey) {
        let mut guard = self.0.lock().await;
        let count = guard.deref_mut().entry(connection).or_insert(0);
        *count += 1;
    }

    pub(crate) async fn decrease(&self, connection: ConnectionKey) {
        let mut guard = self.0.lock().await;
        guard
            .deref_mut()
            .entry(connection)
            .and_modify(|count| *count -= 1);
    }

    pub(crate) async fn copy_inner(&self) -> HashMap<ConnectionKey, usize> {
        let guard = self.0.lock().await;
        guard.deref().clone()
    }
}
