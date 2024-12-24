use crate::errors::{Error, Result};
use crate::registry::JobRegistry;
use figment::providers::{Format, Yaml};
use figment::Figment;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use crate::jobs::Job;

#[derive(Debug, Deserialize, PartialEq)]
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

        if !path.exists() {
            return Err(Error::InvalidConfigPath);
        }

        Ok(Figment::new().merge(Yaml::file(path)).extract()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            main_broker: "http://localhost:9000".to_string(),
            fallback_brokers: vec![
                "http://localhost:9001".to_string(),
            ],
            zone: "dev".to_string(),
            region: "localhost".to_string(),
            batch: 100,
            batch_interval: 10,
            check_config_adapter: ConfigCheckAdapter::Static(StaticConfigAdapter {
                checks: vec![

                ],
            }),
        }
    }
}

pub trait GetJobsRegistry {
    fn get_jobs_registry(&self) -> Result<JobRegistry>;
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "name")]
enum ConfigCheckAdapter {
    #[serde(rename = "static")]
    Static(StaticConfigAdapter),
    #[serde(rename = "file")]
    File(FileConfigCheckAdapter),
}

impl GetJobsRegistry for ConfigCheckAdapter {
    fn get_jobs_registry(&self) -> Result<JobRegistry> {
        match self {
            ConfigCheckAdapter::File(adapter) => adapter.get_jobs_registry(),
            ConfigCheckAdapter::Static(adapter) => adapter.get_jobs_registry(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct FileConfigCheckAdapter {
    path: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct StaticConfigAdapter {
    checks: Vec<Job>,
}

impl GetJobsRegistry for FileConfigCheckAdapter {
    fn get_jobs_registry(&self) -> Result<JobRegistry> {
        JobRegistry::from_configuration_file(PathBuf::from(&self.path))
    }
}

impl GetJobsRegistry for StaticConfigAdapter {
    fn get_jobs_registry(&self) -> Result<JobRegistry> {
        JobRegistry::from_static_config(self.checks.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_example_config() {
        let config_file = Config::from_config_file(env!("CARGO_MANIFEST_DIR").to_string() + "/assets/config/agent.example.yaml")
            .expect("Unable to load default config");
    }
}
