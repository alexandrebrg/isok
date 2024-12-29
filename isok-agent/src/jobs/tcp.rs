use crate::batch_sender::JobResult;
use crate::jobs::{Execute, JobError};
use isok_data::broker_rpc::CheckJobStatus;
use isok_data::JobId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct TcpJob {
    #[serde(default = "JobId::generate")]
    id: JobId,
    endpoint: String,
    interval: u64,
    secured: bool,
    pretty_name: String,
}

impl TcpJob {
    pub fn new(endpoint: String) -> Self {
        TcpJob {
            id: JobId::generate(),
            endpoint,
            interval: 1,
            secured: false,
            pretty_name: "".to_string(),
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }
}

#[async_trait::async_trait]
impl Execute for TcpJob {
    async fn execute(&self, tx: UnboundedSender<JobResult>) -> Result<(), JobError> {
        let mut msg = JobResult::new(self.id());

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

    fn id(&self) -> JobId {
        self.id.clone()
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(self.interval)
    }
}

#[cfg(test)]
mod tests {
    use crate::jobs::tcp::TcpJob;
    use crate::jobs::Execute;
    use isok_data::broker_rpc::CheckJobStatus;
    use isok_data::JobId;

    #[tokio::test]
    async fn test_tcp_job() {
        let tcp = TcpJob {
            id: JobId::generate(),
            endpoint: "toto".to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };

        let tcp2 = TcpJob {
            id: JobId::generate(),
            endpoint: "tata".to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };

        let jobs = vec![tcp, tcp2];
        let a = serde_yaml::to_string(&jobs);
        println!("{:#}", a.unwrap());
    }

    #[tokio::test]
    async fn test_tcp_job_invalid_endpoint() {
        let tcp = TcpJob {
            id: JobId::generate(),
            endpoint: "toto".to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tcp.execute(tx)
            .await
            .expect("Expected execution to succeed");
        let result = rx
            .recv()
            .await
            .expect("Expected to receive unreachable result");

        assert_eq!(result.status, CheckJobStatus::Unreachable);
    }

    #[tokio::test]
    async fn test_tcp_job_valid_endpoint_online() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Unable to bind to port");
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (socket, _) = listener
                .accept()
                .await
                .expect("Unable to accept connection");
        });
        let tcp = TcpJob {
            id: JobId::generate(),
            endpoint: "127.0.0.1".to_string() + ":" + &port.to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tcp.execute(tx)
            .await
            .expect("Expected execution to succeed");
        let result = rx
            .recv()
            .await
            .expect("Expected to receive reachable result");
        assert_eq!(result.status, CheckJobStatus::Reachable);
    }

    #[tokio::test]
    async fn test_tcp_job_valid_endpoint_offline() {
        let tcp = TcpJob {
            id: JobId::generate(),
            endpoint: "127.0.0.1:65534".to_string(),
            interval: 0,
            secured: false,
            pretty_name: "".to_string(),
        };
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tcp.execute(tx)
            .await
            .expect("Expected execution to succeed");
        let result = rx
            .recv()
            .await
            .expect("Expected to receive reachable result");
        assert_eq!(result.status, CheckJobStatus::Unreachable);
    }
}
