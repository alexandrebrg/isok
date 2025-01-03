use crate::batch_sender::BatchSender;
use crate::config::{Config, GetJobsRegistry};
use crate::errors::{Error, Result};
use tokio::join;

mod batch_sender;
pub mod config;
pub mod errors;
pub mod jobs;
mod registry;
mod state;

pub async fn run(config: Config) -> Result<()> {
    let registry = config.get_jobs_registry()?;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut batch_sender = match BatchSender::new(config.result_sender_adapter, rx).await {
        Ok(batch_sender) => batch_sender,
        Err(e) => {
            tracing::error!(
                "Unable to create batch sender, check your configuration. Error: {:?}",
                e
            );
            return Err(Error::UnableToCreateBatchSender(e));
        }
    };
    join!(registry.execute(tx), batch_sender.run());

    Ok(())
}
