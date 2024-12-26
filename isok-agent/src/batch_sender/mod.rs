use enum_dispatch::enum_dispatch;
use futures::SinkExt;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Instant;
use isok_data::broker_rpc::broker_client::BrokerClient;
use isok_data::broker_rpc::{BrokerGrpcClient, CheckJobStatus, CheckResult, HealthResponse};
use crate::config::{BrokerConfig, ResultSenderAdapter};

#[derive(Debug)]
pub struct JobResult {
    pub id: String,
    pub run_at: Instant,
    pub status: CheckJobStatus,
}

impl JobResult {
    pub fn new(id: String) -> Self {
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
    pub async fn new(adapter_cfg: ResultSenderAdapter, rx: UnboundedReceiver<JobResult>) -> Result<Self, BatchSenderError> {
        let connector = match adapter_cfg {
            ResultSenderAdapter::Stdout => BatchSenderType::Stdout(StdoutBatchSender::default()),
            ResultSenderAdapter::Broker(config) => BatchSenderType::Broker(BrokerBatchSender::new(config).await?),
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
}

#[enum_dispatch(BatchSenderOutput)]
pub enum BatchSenderType {
    Stdout(StdoutBatchSender),
    Broker(BrokerBatchSender),
}

#[enum_dispatch]
pub trait BatchSenderOutput {
    async fn send(&mut self, job_result: JobResult) -> Result<(), BatchSenderError>;

    async fn health_check(&mut self) -> Result<(), BatchSenderError>;
}

#[derive(Default)]
pub struct StdoutBatchSender {}

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
    batch: usize,
    batch_interval: usize,
}

impl BrokerBatchSender {
    pub async fn new(config: BrokerConfig) -> Result<Self, BatchSenderError> {
        let client = BrokerClient::connect(config.main_broker.clone()).await
            .map_err(|_| BatchSenderError::InvalidBrokerEndpointConfiguration)?;
        Ok(BrokerBatchSender {
            client,
            backlog: vec![],
            broker: config.main_broker,
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
            tags: Default::default(),
            created_at: None,
            events: vec![CheckResult {
                check_uuid: job_result.id.clone(),
                run_at: None,
                status: job_result.status.into(),
                metrics: Default::default(),
                tags: None,
                details: Default::default(),
            }],
        };
        self.client.batch_send(request).await.map_err(|e| BatchSenderError::UnableToSendBatch(e.to_string()))?;
        Ok(())
    }

    async fn health_check(&mut self) -> Result<(), BatchSenderError> {
        match self.client.health(isok_data::broker_rpc::HealthRequest {}).await {
            Ok(response) => {
                if !response.get_ref().healthy {
                    return Err(BatchSenderError::BrokerUnhealthy);
                }
                Ok(())
            }
            Err(_) => Err(BatchSenderError::BrokerUnhealthy)
        }
    }
}