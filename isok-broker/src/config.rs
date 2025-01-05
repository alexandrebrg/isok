use crate::api::ApiError;
use crate::message_broker::MessageBrokerError;
use figment::providers::{Format, Yaml};
use figment::Figment;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub kafka: KafkaConfig,
    pub api: ApiConfig,
}

#[derive(Deserialize, Clone)]
pub struct KafkaConfig {
    pub topic: String,
    pub properties: HashMap<String, String>,
}

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    pub listen_address: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to load config file: {0}")]
    UnableToLoadConfigFile(#[from] figment::Error),
    #[error("Unable to create message broker: {0}")]
    UnableToCreateMessageBroker(#[from] MessageBrokerError),
    #[error("Unable to start API server")]
    UnableToStartApiServer(#[from] ApiError),
}

impl Config {
    pub fn from_config_file(path: impl Into<PathBuf>) -> Result<Self, Error> {
        Ok(Figment::new().merge(Yaml::file(path.into())).extract()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            kafka: KafkaConfig {
                topic: "isok.agent.results".to_string(),
                properties: HashMap::from([(
                    "bootstrap.servers".to_string(),
                    "localhost:9092".to_string(),
                )]),
            },
            api: ApiConfig {
                listen_address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            },
        }
    }
}
