use std::error::Error;
use enum_dispatch::enum_dispatch;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Instant;
use isok_data::{CheckBatchRequest, CheckJobStatus};

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
    rx: UnboundedReceiver<JobResult>,
}

impl BatchSender {
    pub fn new(rx: UnboundedReceiver<JobResult>) -> Self {
        BatchSender { rx }
    }

    pub async fn run(&mut self) {
        while let Some(job) = self.rx.recv().await {
            tracing::debug!(job_id = ?job.id, job_status = ?job.status, "Job result");
        }
    }
}