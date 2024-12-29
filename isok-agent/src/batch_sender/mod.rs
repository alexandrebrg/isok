use crate::config::{BrokerConfig, ResultSenderAdapter, SocketConfig};
use enum_dispatch::enum_dispatch;
use isok_data::broker_rpc::broker_client::BrokerClient;
use isok_data::broker_rpc::{BrokerGrpcClient, CheckJobStatus, CheckResult, Tags};
use isok_data::JobId;
use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Instant;

#[derive(Debug)]
pub struct JobResult {
    pub id: JobId,
    pub run_at: Instant,
    pub status: CheckJobStatus,
}

impl JobResult {
    pub fn new(id: JobId) -> Self {
        JobResult {
            id,
            run_at: Instant::now(),
            status: CheckJobStatus::Unknown,
        }
    }

    pub(crate) fn set_status(&mut self, status: CheckJobStatus) {
        self.status = status;
    }
}

pub struct BatchSender {
    connector: BatchSenderType,
    rx: UnboundedReceiver<JobResult>,
}

impl BatchSender {
    pub async fn new(
        adapter_cfg: ResultSenderAdapter,
        rx: UnboundedReceiver<JobResult>,
    ) -> Result<Self, BatchSenderError> {
        let connector = match adapter_cfg {
            ResultSenderAdapter::Stdout => BatchSenderType::Stdout(StdoutBatchSender::new()),
            ResultSenderAdapter::Broker(config) => {
                BatchSenderType::Broker(BrokerBatchSender::new(config).await?)
            }
            ResultSenderAdapter::Socket(config) => {
                BatchSenderType::Socket(SocketBatchSender::new(config).await?)
            }
        };
        Ok(BatchSender { rx, connector })
    }

    pub async fn run(&mut self) {
        while let Some(job) = self.rx.recv().await {
            match self.connector.send(job).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Unable to send job result {:?}", e);
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BatchSenderError {
    #[error("Provided Broker URL is either invalid or unreachable")]
    InvalidBrokerEndpointConfiguration,
    #[error("Broker is not healthy")]
    BrokerUnhealthy,
    #[error("Unable to send job result: {0}")]
    UnableToSendBatch(String),
    #[error("Unable to forward job result to socket: {0}")]
    WriteSocketError(String),
    #[error("Unable to connect to socket: {0}")]
    OpenSocketError(String),
}

#[enum_dispatch(BatchSenderOutput)]
pub enum BatchSenderType {
    Stdout(StdoutBatchSender),
    Broker(BrokerBatchSender),
    Socket(SocketBatchSender),
}

#[enum_dispatch]
pub trait BatchSenderOutput {
    async fn send(&mut self, job_result: JobResult) -> Result<(), BatchSenderError>;

    async fn health_check(&mut self) -> Result<(), BatchSenderError>;
}

pub struct SocketBatchSender {
    stream: UnixStream,
}

impl SocketBatchSender {
    pub async fn new(config: SocketConfig) -> Result<Self, BatchSenderError> {
        let stream = UnixStream::connect(config.path)
            .await
            .map_err(|e| BatchSenderError::OpenSocketError(e.to_string()))?;
        Ok(Self { stream })
    }
}

impl BatchSenderOutput for SocketBatchSender {
    async fn send(&mut self, job_result: JobResult) -> Result<(), BatchSenderError> {
        let request = isok_data::broker_rpc::CheckBatchRequest {
            created_at: None,
            tags: Some(Tags {
                agent_id: "local-agent".to_string(),
                zone: "dev".to_string(),
                region: "localhost".to_string(),
            }),
            events: vec![CheckResult {
                id_ulid: job_result.id.clone().to_string(),
                run_at: None,
                status: job_result.status.into(),
                metrics: Default::default(),
                tags: None,
                details: Default::default(),
            }],
        };
        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        // We first write the length of the message into 8 bytes (more than necessary),
        // then the message itself
        let len = buffer.len();
        let mut len_bytes = [0u8; 8];
        len_bytes = len.to_be_bytes();
        self.stream
            .write_all(&len_bytes)
            .await
            .map_err(|e| BatchSenderError::WriteSocketError(e.to_string()))?;

        self.stream
            .write_all(&buffer)
            .await
            .map_err(|e| BatchSenderError::WriteSocketError(e.to_string()))?;
        Ok(())
    }

    async fn health_check(&mut self) -> Result<(), BatchSenderError> {
        Ok(())
    }
}

pub struct StdoutBatchSender {}

impl StdoutBatchSender {
    pub fn new() -> Self {
        Self {}
    }
}

impl BatchSenderOutput for StdoutBatchSender {
    async fn send(&mut self, job_result: JobResult) -> Result<(), BatchSenderError> {
        tracing::info!(job_id = ?job_result.id, job_status = ?job_result.status, "Job result");
        Ok(())
    }

    async fn health_check(&mut self) -> Result<(), BatchSenderError> {
        Ok(())
    }
}

pub struct BrokerBatchSender {
    client: BrokerGrpcClient,
    broker: String,
    backlog: Vec<JobResult>,
    zone: String,
    region: String,
    agent_id: String,
    batch: usize,
    batch_interval: usize,
}

impl BrokerBatchSender {
    pub async fn new(config: BrokerConfig) -> Result<Self, BatchSenderError> {
        let client = BrokerClient::connect(config.main_broker.clone())
            .await
            .map_err(|_| BatchSenderError::InvalidBrokerEndpointConfiguration)?;
        Ok(BrokerBatchSender {
            client,
            backlog: vec![],
            broker: config.main_broker,
            agent_id: config.agent_id,
            zone: config.zone,
            region: config.region,
            batch: config.batch,
            batch_interval: config.batch_interval,
        })
    }
}

impl BatchSenderOutput for BrokerBatchSender {
    async fn send(&mut self, job_result: JobResult) -> Result<(), BatchSenderError> {
        let request = isok_data::broker_rpc::CheckBatchRequest {
            created_at: None,
            tags: Some(Tags {
                agent_id: self.agent_id.clone(),
                zone: self.zone.clone(),
                region: self.region.clone(),
            }),
            events: vec![CheckResult {
                id_ulid: job_result.id.clone().to_string(),
                run_at: None,
                status: job_result.status.into(),
                metrics: Default::default(),
                tags: None,
                details: Default::default(),
            }],
        };
        self.client
            .batch_send(request)
            .await
            .map_err(|e| BatchSenderError::UnableToSendBatch(e.to_string()))?;
        Ok(())
    }

    async fn health_check(&mut self) -> Result<(), BatchSenderError> {
        match self
            .client
            .health(isok_data::broker_rpc::HealthRequest {})
            .await
        {
            Ok(response) => {
                if !response.get_ref().healthy {
                    return Err(BatchSenderError::BrokerUnhealthy);
                }
                Ok(())
            }
            Err(_) => Err(BatchSenderError::BrokerUnhealthy),
        }
    }
}
