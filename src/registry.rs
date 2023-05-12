use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::config::Config;

#[derive(Clone, Debug, Default)]
pub(crate) struct ConnectionRegistry(Arc<Mutex<HashMap<Config, usize>>>);

impl ConnectionRegistry {
    pub(crate) async fn increase(&self, config: Config) {
        let mut guard = self.0.lock().await;
        let count = guard.deref_mut().entry(config).or_insert(0);
        *count += 1;
    }

    pub(crate) async fn decrease(&self, config: Config) {
        let mut guard = self.0.lock().await;
        guard
            .deref_mut()
            .entry(config)
            .and_modify(|count| *count -= 1);
    }

    pub(crate) async fn copy_inner(&self) -> HashMap<Config, usize> {
        let guard = self.0.lock().await;
        guard.deref().clone()
    }
}
