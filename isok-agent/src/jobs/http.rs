use crate::jobs::Execute;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpJob {
    endpoint: String,
    pretty_name: String,
    interval: u64,
    headers: HashMap<String, String>,
}

impl HttpJob {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            pretty_name: "".to_string(),
            interval: 0,
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        }
    }
}

#[async_trait::async_trait]
impl Execute for HttpJob {
    async fn execute(&self, tx: UnboundedSender<String>) -> Result<(), Box<dyn Error>> {
        tx.send(format!("http: {}", self.endpoint.clone()))?;
        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(self.interval)
    }
}
