use crate::config::{Config, GetJobsRegistry};
use crate::errors::Result;
use tokio::join;

mod brokers;
mod config;
mod errors;
mod jobs;
mod registry;
mod state;

pub async fn run() -> Result<()> {
    let config = Config::from_config_file(
        "/home/noa/Documents/Travail/Clever/isok/isok-agent/assets/config/agent.example.yaml",
    )?;
    let registry = config.get_jobs_registry()?;

    join!(registry.execute(), registry.run());

    Ok(())
}

#[tokio::test]
async fn test_run() {
    run().await;
}
