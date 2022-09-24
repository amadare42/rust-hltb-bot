use std::str::FromStr;
use log::{LevelFilter};
use simple_logger::SimpleLogger;
use crate::model::RunMode;

mod model;
mod formatting;
mod api_client;
mod telegram;

mod tests;
mod lambda;

#[tokio::main]
async fn main() {
    configure_logging();

    let run_mode = std::env::var("RUN_MODE")
        .map_or_else(|_| {
            log::warn!("RUN_MODE missing or invalid");
            RunMode::WebHook
        }, |run_mode| RunMode::from_str(&run_mode).unwrap());

    log::info!("Running in {:?} mode", run_mode);

    match run_mode {
        RunMode::Polling => telegram::run_polling().await.unwrap(),
        RunMode::WebHook => lambda::run().await.unwrap(),
    }
}

fn configure_logging() {
    let log_level = std::env::var("LOG_LEVEL").ok()
        .and_then(|log_level| LevelFilter::from_str(&log_level).ok())
        .unwrap_or(LevelFilter::Info);

    SimpleLogger::new().with_level(log_level).init().unwrap();
}