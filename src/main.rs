



mod config;
mod handler;
mod serve;
mod utils;

use crate::{config::Config, serve::serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::setup_logger();

    let configs = Config::from_file("config.json").await?;
    let handles: Vec<_> = configs
        .into_iter()
        .map(move |config| tokio::spawn(serve(config)))
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
