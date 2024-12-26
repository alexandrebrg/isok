use crate::batch_sender::JobResult;
use crate::jobs::http::HttpJob;
use crate::jobs::tcp::TcpJob;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

mod http;
mod tcp;

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
    fn interval(&self) -> Duration;
}

#[cfg(test)]
mod tests {
    use crate::jobs::http::HttpJob;
    use crate::jobs::tcp::TcpJob;
    use crate::jobs::Job;

    #[tokio::test]
    async fn test_ser() {
        let jobs = vec![
            Job::Tcp(TcpJob::new("toto".to_string())),
            Job::Http(HttpJob::new("tata".to_string())),
        ];
        let a = serde_yaml::to_string(&jobs).unwrap();
        println!("{}", a);
        let b: Vec<Job> = serde_yaml::from_str(&a).unwrap();
        dbg!(b);
    }
}
