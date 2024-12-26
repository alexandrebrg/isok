mod api;
mod message_broker;

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use figment::Figment;
use figment::providers::{Format, Yaml};
use serde::Deserialize;
use crate::message_broker::{KafkaMessageBroker, MessageBroker, MessageBrokerError};

pub async fn run(config: Config) -> Result<(), Error> {
    let message_broker = KafkaMessageBroker::try_new(config.kafka)
        .inspect_err(|e| tracing::error!("Unable to create message broker: {:?}", e))?;

    match api::BrokerGrpcService::new(MessageBroker::Kafka(message_broker)).run_on(config.api.listen_address).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Unable to start API server: {:?}", e);
        }
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct Config {
    kafka: KafkaConfig,
    api: ApiConfig,
}

#[derive(Deserialize)]
pub struct KafkaConfig {
    topic: String,
    properties: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ApiConfig {
    listen_address: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to load config file: {0}")]
    UnableToLoadConfigFile(#[from] figment::Error),
    #[error("Unable to create message broker: {0}")]
    UnableToCreateMessageBroker(#[from] MessageBrokerError),
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
                properties: HashMap::from([("bootstrap.servers".to_string(), "localhost:9092".to_string())]),
            },
            api: ApiConfig {
                listen_address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            },
        }
    }
}