use crate::config::{Config, GetJobsRegistry};
use crate::errors::Result;
use tokio::join;
use crate::batch_sender::BatchSender;

mod brokers;
pub mod config;
pub mod errors;
mod jobs;
mod registry;
mod state;
mod batch_sender;

pub async fn run(config: Config) -> Result<()> {
    let registry = config.get_jobs_registry()?;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut batch_sender = BatchSender::new(rx);
    join!(registry.execute(tx), batch_sender.run());

    Ok(())
}
