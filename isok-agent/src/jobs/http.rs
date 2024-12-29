use crate::batch_sender::JobResult;
use crate::jobs::{Execute, JobError};
use isok_data::broker_rpc::CheckJobStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use isok_data::JobId;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct HttpJob {
    #[serde(default = "JobId::generate")]
    id: JobId,
    endpoint: String,
    pretty_name: String,
    interval: u64,
    headers: HashMap<String, String>,
}

impl HttpJob {
    pub fn new(endpoint: String) -> Self {
        Self {
            id: JobId::generate(),
            endpoint,
            pretty_name: "".to_string(),
            interval: 0,
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        }
    }
}

#[async_trait::async_trait]
impl Execute for HttpJob {
    async fn execute(&self, tx: UnboundedSender<JobResult>) -> Result<(), JobError> {
        let mut msg = JobResult::new(self.id());
        let client = reqwest::Client::new();
        match client.get(&self.endpoint).send().await {
            Ok(response) => {
                let status = response.status();
                msg.set_status(CheckJobStatus::Reachable);
            }
            Err(e) => {
                msg.set_status(CheckJobStatus::Unreachable);
            }
        }
        if let Err(e) = tx.send(msg) {
            tracing::error!("Unable to send message to channel {}", e);
        }
        Ok(())
    }

    fn pretty_name(&self) -> String {
        self.pretty_name.clone()
    }

    fn id(&self) -> JobId {
        self.id.clone()
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(self.interval)
    }
}
