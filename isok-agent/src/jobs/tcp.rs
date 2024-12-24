use crate::jobs::{Execute, JobError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use isok_data::CheckJobStatus;
use crate::batch_sender::JobResult;
use crate::registry::JobRegistry;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
    async fn execute(&self, tx: UnboundedSender<JobResult>) -> Result<(), JobError> {
        let mut msg = JobResult::new(self.pretty_name.clone());

        let addr = SocketAddr::from_str(&self.endpoint);
        if let Ok(addr) = addr {
            match TcpStream::connect(addr).await {
                Ok(_) => {
                    msg.set_status(CheckJobStatus::Reachable);
                }
                Err(_) => {
                    msg.set_status(CheckJobStatus::Unreachable);
                }
            }
        } else {
            msg.set_status(CheckJobStatus::Unreachable);
        }

        if let Err(e) = tx.send(msg) {
            tracing::error!("Unable to send message to channel {}", e);
        }
        Ok(())
    }

    fn pretty_name(&self) -> String {
        self.pretty_name.clone()
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
