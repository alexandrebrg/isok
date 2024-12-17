use crate::jobs::Execute;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Deserialize, Serialize)]
pub struct TcpJob {
    endpoint: String,
    interval: u64,
    secured: bool,
    pretty_name: String,
}

impl TcpJob {
    pub fn new(endpoint: String) -> Self {
        TcpJob {
            endpoint,
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Execute for TcpJob {
    async fn execute(&self, tx: UnboundedSender<String>) -> Result<(), Box<dyn Error>> {
        tx.send(format!("tcp: {}", self.endpoint.clone()))?;
        Ok(())
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(self.interval)
    }
}

#[cfg(test)]
mod tests {
    use crate::jobs::tcp::TcpJob;

    #[tokio::test]
    async fn test_tcp_job() {
        let tcp = TcpJob {
            endpoint: "toto".to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };

        let tcp2 = TcpJob {
            endpoint: "tata".to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };

        let jobs = vec![tcp, tcp2];
        let a = serde_yaml::to_string(&jobs);
        println!("{:#}", a.unwrap());
    }
}
