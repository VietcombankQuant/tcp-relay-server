mod config;
mod handler;
mod registry;
mod serve;
mod utils;

use std::time::Duration;

use crate::{config::Config, registry::ConnectionRegistry, serve::serve};

async fn monitor_registry(registry: ConnectionRegistry) {
    loop {
        let counters = registry.copy_inner().await;
        println!("Hello World {:#?}", counters);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::setup_logger();

    let configs = Config::from_file("config.json").await?;
    let registry = ConnectionRegistry::default();
    tokio::spawn(monitor_registry(registry.clone()));

    let handles: Vec<_> = configs
        .into_iter()
        .map(move |config| (config, registry.clone()))
        .map(|(config, registry)| tokio::spawn(serve(config, registry.clone())))
        .collect();

    for handle in handles {
        match handle.await {
            Ok(_) => {}
            Err(err) => {
                log::error!(
                    "Error occured while joining relay server {}",
                    err.to_string()
                );
            }
        }
    }

    Ok(())
}
