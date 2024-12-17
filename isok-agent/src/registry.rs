use crate::errors::Result;
use crate::jobs::{Execute, Job};
use crate::state::JobState;
use figment::providers::{Format, Yaml};
use figment::Figment;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use surf::http::Method;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct JobRegistry {
    inner: BTreeMap<u64, JobState>,
    last_id: u64,
    tx: UnboundedSender<String>,
    rx: RwLock<UnboundedReceiver<String>>,
}

impl JobRegistry {
    pub fn from_configuration_file(path_buf: PathBuf) -> Result<JobRegistry> {
        #[derive(Debug, Deserialize)]
        struct Wrapper {
            checks: Vec<Job>,
        }

        let mut registry = JobRegistry::new();

        let jobs = Figment::new()
            .merge(Yaml::file(path_buf))
            .extract::<Wrapper>()?;

        for job in jobs.checks {
            registry.append(job);
        }

        Ok(registry)
    }
}

impl JobRegistry {
    pub(crate) fn new() -> JobRegistry {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        JobRegistry {
            inner: Default::default(),
            last_id: 0,
            tx,
            rx: RwLock::new(rx),
        }
    }

    fn append(&mut self, job: Job) {
        self.inner.insert(self.last_id, JobState::new(job));
        self.last_id += 1;
    }

    pub(crate) async fn execute(&self) {
        loop {
            let tx = self.tx.clone();
            for (&_id, job) in &self.inner {
                if let Some(last_run) = job.last_run() {
                    // todo: cool down job
                } else {
                    job.execute(tx.clone()).await.unwrap();
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    pub(crate) async fn run(&self) {
        while let Some(x) = self.rx.write().await.recv().await {
            surf::client()
                .request(Method::Get, format!("http://localhost:8000/{}", x))
                .send()
                .await
                .unwrap();
        }
    }
}
