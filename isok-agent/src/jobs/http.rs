use crate::batch_sender::JobResult;
use crate::jobs::{Execute, JobError};
use isok_data::broker_rpc::CheckJobStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct HttpJob {
    endpoint: String,
    headers: HashMap<String, String>,
}

impl HttpJob {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        }
    }
}

#[async_trait]
impl Execute for HttpJob {
    async fn execute(&self, msg: &mut JobResult) -> Result<(), JobError> {
        let client = reqwest::Client::new();
        match client.get(&self.endpoint).send().await {
            Ok(response) => {
                let _ = response.status();
                msg.set_status(CheckJobStatus::Reachable);
            }
            Err(_) => {
                msg.set_status(CheckJobStatus::Unreachable);
            }
        }
        Ok(())
    }

}
