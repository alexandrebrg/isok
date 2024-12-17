use crate::errors::Result;
use crate::registry::JobRegistry;
use figment::providers::{Format, Yaml};
use figment::Figment;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    main_broker: String,
    fallback_brokers: Vec<String>,
    zone: String,
    region: String,
    batch: usize,
    batch_interval: usize,
    check_config_adapter: ConfigCheckAdapter,
}

impl GetJobsRegistry for Config {
    fn get_jobs_registry(&self) -> Result<JobRegistry> {
        self.check_config_adapter.get_jobs_registry()
    }
}

impl Config {
    pub fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        Ok(Figment::new().merge(Yaml::file(path)).extract()?)
    }
}

pub trait GetJobsRegistry {
    fn get_jobs_registry(&self) -> Result<JobRegistry>;
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name")]
enum ConfigCheckAdapter {
    #[serde(rename = "file")]
    File(FileConfigCheckAdapter),
}

impl GetJobsRegistry for ConfigCheckAdapter {
    fn get_jobs_registry(&self) -> Result<JobRegistry> {
        match self {
            ConfigCheckAdapter::File(adapter) => adapter.get_jobs_registry(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct FileConfigCheckAdapter {
    path: String,
}

impl GetJobsRegistry for FileConfigCheckAdapter {
    fn get_jobs_registry(&self) -> Result<JobRegistry> {
        JobRegistry::from_configuration_file(PathBuf::from(&self.path))
    }
}
