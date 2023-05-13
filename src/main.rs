mod book_keeper;
mod config;
mod handler;
mod serve;
mod utils;

use crate::{book_keeper::ConnBookKeeper, config::Config, serve::serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::setup_logger();

    let configs = Config::from_file("config.json").await?;
    let (connection_book, event_sender) = ConnBookKeeper::new();
    tokio::spawn(connection_book.process_events());

    let handles: Vec<_> = configs
        .into_iter()
        .map(move |config| (config, event_sender.clone()))
        .map(|(config, event_sender)| tokio::spawn(serve(config, event_sender.clone())))
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
