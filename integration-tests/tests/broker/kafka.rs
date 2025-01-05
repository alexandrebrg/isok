use integration_tests::{BrokerTestingRunner, TRACING};
use isok_data::broker_rpc::broker_client::BrokerClient;
use isok_data::broker_rpc::CheckBatchRequest;
use once_cell::sync::Lazy;

#[tokio::test]
async fn test_kafka_message_integrity() {
    Lazy::force(&TRACING);
    let broker = BrokerTestingRunner::new().start_broker();

    let mut client = BrokerClient::connect(format!("http://localhost:{}", broker.listening_port))
        .await
        .expect("Failed to connect to broker");
    // @AlexandreBrg: Whenever using the mock cluster, and the batch size is at most 1,
    // the producer will only be produced (or consumer consume?) after a few seconds,
    // with at least 2 messages. Since MockCluster is an tech preview stuff, and librdkafka
    // is not yet stable on this, I assume it could be a bug, or misunderstanding of the
    // properties.
    tokio::spawn(async move {
        loop {
            let batch = CheckBatchRequest {
                tags: Some(isok_data::broker_rpc::Tags {
                    agent_id: "test".to_string(),
                    zone: "dev".to_string(),
                    region: "localhost".to_string(),
                }),
                events: vec![isok_data::broker_rpc::CheckResult {
                    id_ulid: "test".to_string(),
                    run_at: None,
                    status: isok_data::broker_rpc::CheckJobStatus::Reachable as i32,
                    metrics: Default::default(),
                    tags: None,
                    details: Default::default(),
                }],
                created_at: None,
            };
            client
                .batch_send(batch)
                .await
                .expect("Failed to send batch");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let consumer = broker.get_topic_consumer();
    let msg = consumer.recv().await.expect("Expected message");
}
