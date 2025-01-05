mod api;
pub mod config;
mod message_broker;

use crate::config::{Config, Error};
use crate::message_broker::{KafkaMessageBroker, MessageBroker};

pub async fn run(config: Config) -> Result<(), Error> {
    let message_broker = KafkaMessageBroker::try_new(config.kafka)?;

    api::BrokerGrpcService::new(MessageBroker::Kafka(message_broker))
        .run_on(config.api.listen_address)
        .await
        .map_err(Error::UnableToStartApiServer)?;
    Ok(())
}
