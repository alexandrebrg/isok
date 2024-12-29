use crate::batch_sender::JobResult;
use crate::jobs::http::HttpJob;
use crate::jobs::tcp::TcpJob;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use isok_data::JobId;

pub mod http;
pub mod tcp;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Job {
    #[serde(rename = "tcp")]
    Tcp(TcpJob),
    #[serde(rename = "http")]
    Http(HttpJob),
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Invalid job config {0}")]
    InvalidJobConfig(String),
    #[error("Unable to execute job {0}")]
    HttpError(#[from] reqwest::Error),
}

#[async_trait::async_trait]
impl Execute for Job {
    async fn execute(&self, tx: UnboundedSender<JobResult>) -> Result<(), JobError> {
        match self {
            Job::Tcp(job) => job.execute(tx).await,
            Job::Http(job) => job.execute(tx).await,
        }
    }

    fn pretty_name(&self) -> String {
        match self {
            Job::Tcp(job) => job.pretty_name(),
            Job::Http(job) => job.pretty_name(),
        }
    }

    fn id(&self) -> JobId {
        match self {
            Job::Tcp(job) => job.id(),
            Job::Http(job) => job.id(),
        }
    }

    fn interval(&self) -> Duration {
        match self {
            Job::Tcp(job) => job.interval(),
            Job::Http(job) => job.interval(),
        }
    }
}

#[async_trait::async_trait]
pub trait Execute {
    async fn execute(&self, tx: UnboundedSender<JobResult>) -> Result<(), JobError>;
    fn pretty_name(&self) -> String;
    fn id(&self) -> JobId;
    fn interval(&self) -> Duration;
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use isok_data::JobId;
    use crate::jobs::http::HttpJob;
    use crate::jobs::tcp::TcpJob;
    use crate::jobs::{Execute, Job};

    #[derive(Debug, Deserialize, Serialize)]
    struct DummyRootJob {
        jobs: Vec<Job>,
    }

    #[test]
    fn test_job_id_serde_generate() {
        let config = r#"
        jobs:
            - type: "http"
              pretty_name: "5s failing endpoint"
              endpoint: "https://my_endpoint.com/api/v1/healthy?system_only=true"
              interval: 5
              headers:
                Authorization: "Bearer..."
            "#;
        let root: DummyRootJob = serde_yaml::from_str(config).unwrap();
        match &root.jobs[0] {
            Job::Http(job) => {
                assert_eq!(job.id().to_string().len(), 26);
            }
            _ => panic!("Expected to be able to deserialize job"),
        }
    }

    #[test]
    fn test_job_id_serde_from_ulid() {
        let config = r#"
        jobs:
            - type: "http"
              id: "01ARZ3NDEKTSV4RRWETS2EGZ5M"
              pretty_name: "5s failing endpoint"
              endpoint: "https://my_endpoint.com/api/v1/healthy?system_only=true"
              interval: 5
              headers:
                Authorization: "Bearer..."
            "#;
        let root: DummyRootJob = serde_yaml::from_str(config).unwrap();
        match &root.jobs[0] {
            Job::Http(job) => {
                assert_eq!(job.id().to_string(), "01ARZ3NDEKTSV4RRWETS2EGZ5M");
            }
            _ => panic!("Expected to be able to deserialize job"),
        }
    }
}
