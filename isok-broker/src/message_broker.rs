use crate::KafkaConfig;
use isok_data::broker_rpc::CheckResult;
use prost::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum MessageBrokerError {
    #[error("Unable to create producer: {0}")]
    UnableToCreateProducer(#[from] rdkafka::error::KafkaError),
    #[error("The message broker isn't able to process any message")]
    ServiceUnhealthy,
    #[error("A batch or single check result couldn't be stored persistently: {0}")]
    UnableToStoreCheckResult(String),
}

#[enum_dispatch::enum_dispatch(MessageBrokerSender)]
pub enum MessageBroker {
    Kafka(KafkaMessageBroker),
}

pub struct KafkaMessageBroker {
    producer: FutureProducer,
    topic: String,
}

#[enum_dispatch::enum_dispatch]
pub trait MessageBrokerSender {
    async fn process_batch(&self, batch: &[CheckResult]) -> Result<(), MessageBrokerError> {
        for message in batch {
            self.process_message(message).await?;
        }
        Ok(())
    }
    async fn process_message(&self, message: &CheckResult) -> Result<(), MessageBrokerError>;
    async fn health_check(&self) -> Result<(), MessageBrokerError>;
}

impl MessageBrokerSender for KafkaMessageBroker {
    async fn process_message(&self, message: &CheckResult) -> Result<(), MessageBrokerError> {
        let mut buffer = Vec::new();
        message.encode(&mut buffer).unwrap();
        let record = FutureRecord::to(&self.topic)
            .payload(&buffer)
            .key(&message.check_uuid);

        self.producer
            .send(record, Duration::from_secs(2))
            .await
            .map_err(|e| MessageBrokerError::UnableToStoreCheckResult(format!("{:?}", e)))?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), MessageBrokerError> {
        Ok(())
    }
}

impl KafkaMessageBroker {
    pub fn try_new(config: KafkaConfig) -> Result<Self, MessageBrokerError> {
        let topic = config.topic.clone();
        let producer = FutureProducer::try_from(config)?;
        Ok(KafkaMessageBroker { producer, topic })
    }
}

impl TryFrom<KafkaConfig> for FutureProducer {
    type Error = MessageBrokerError;

    fn try_from(value: KafkaConfig) -> Result<Self, Self::Error> {
        let mut client = ClientConfig::new();
        client.set("message.timeout.ms", "5000");

        for (key, value) in value.properties {
            client.set(key, value);
        }

        client
            .create()
            .map_err(|e| MessageBrokerError::UnableToCreateProducer(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isok_data::broker_rpc::{CheckBatchRequest, CheckJobStatus};
    use prost::Message;
    use rdkafka::consumer::{Consumer, StreamConsumer};
    use rdkafka::mocking::MockCluster;
    use rdkafka::Message as KafkaMessage;
    use std::collections::HashMap;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_kafka_message_integrity() {
        let topic = "test2";
        let mock_cluster = MockCluster::new(3).unwrap();

        mock_cluster
            .create_topic(topic, 32, 3)
            .expect("Failed to create topic");

        let config = KafkaConfig {
            topic: topic.to_string(),
            properties: HashMap::from([(
                "bootstrap.servers".to_string(),
                mock_cluster.bootstrap_servers(),
            )]),
        };

        let kafka =
            KafkaMessageBroker::try_new(config).expect("Failed to create Kafka message broker");
        let batch = vec![CheckResult {
            check_uuid: "test".to_string(),
            run_at: None,
            status: CheckJobStatus::Reachable.into(),
            metrics: Default::default(),
            tags: None,
            details: Default::default(),
        }];

        let batch_thread = batch.clone();
        tokio::spawn(async move {
            // @AlexandreBrg: There is an issue with the mocked cluster,
            // if send <100k messages, the consumer will not receive any message.
            // I personally think it's linked to queue buffering properties, tried
            // multiple things to fix it, but nothing worked.
            // Related issue: https://github.com/fede1024/rust-rdkafka/issues/629
            let mut i = 0_usize;
            loop {
                kafka
                    .process_batch(&batch_thread)
                    .await
                    .expect("Failed to process batch");
                i += 1;
            }
        });

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", mock_cluster.bootstrap_servers())
            .set("group.id", "test_kafka_message_integrity")
            .create()
            .expect("Consumer creation failed");

        consumer
            .subscribe(&[topic])
            .expect("Can't subscribe to specified topics");

        let msg = consumer.recv().await.expect("Expected message");
        let payload = msg.payload().expect("Expected payload");

        let mut message = CheckResult::decode(payload).expect("Expected decode to succeed");
        assert_eq!(message, batch[0]);
    }
}
