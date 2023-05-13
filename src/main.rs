mod book_keeper;
mod config;
mod handler;
mod serve;
mod utils;

use crate::{book_keeper::ConnBookKeeper, config::Config, serve::serve};

#[tokio::main]
async fn main() {
    utils::setup_logger();

    let configs = match Config::from_file("config.json").await {
        Ok(configs) => configs,
        Err(err) => {
            log::error!(
                "Failed to read from config file with error {}",
                err.to_string()
            );
            return;
        }
    };

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
}
