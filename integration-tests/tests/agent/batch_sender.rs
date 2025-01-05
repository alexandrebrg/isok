use integration_tests::{AgentTestingRunner, NEXT_PORT, TRACING};
use isok_agent::jobs::http::HttpJob;
use isok_agent::jobs::tcp::TcpJob;
use isok_agent::jobs::{Job, JobInnerConfig};
use isok_data::broker_rpc::CheckBatchRequest;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::time::Duration;

#[tokio::test]
async fn test_agent_feedback_ok_http() {
    Lazy::force(&TRACING);
    let http_config = JobInnerConfig::Http(HttpJob::new("https://google.com".to_string()));
    let check = Job::new(Duration::from_secs(10), http_config, "google".to_string());
    let expected_id = check.id().to_string();

    let socket_path = tempfile::NamedTempFile::new().unwrap().path().to_path_buf();
    let (mut rx, _) =
        AgentTestingRunner::create_path_socket_listener::<CheckBatchRequest>(socket_path.clone());

    let agent = AgentTestingRunner::new()
        .add_check(check.clone())
        .use_socket_sender(socket_path.clone())
        .run();

    let check_result = rx.recv().await.unwrap();
    agent.abort();
    let rcv_check = check_result
        .events
        .first()
        .expect("Expected to receive at least one check result");
    assert_eq!(expected_id, rcv_check.id_ulid);
    assert_eq!(
        rcv_check.status,
        isok_data::broker_rpc::CheckJobStatus::Reachable as i32
    );
}

#[tokio::test]
async fn test_agent_feedback_ok_tcp() {
    let port = NEXT_PORT.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u16;
    let (_, _) = AgentTestingRunner::create_tcp_socket_listener::<CheckBatchRequest>(port);
    let tcp_config = JobInnerConfig::Tcp(TcpJob::new(format!("127.0.0.1:{}", port)));
    let check = Job::new(Duration::from_secs(10), tcp_config, "tcp".to_string());

    let socket_path = tempfile::NamedTempFile::new().unwrap().path().to_path_buf();
    let (mut rx, _) =
        AgentTestingRunner::create_path_socket_listener::<CheckBatchRequest>(socket_path.clone());

    let agent = AgentTestingRunner::new()
        .add_check(check.clone())
        .use_socket_sender(socket_path.clone())
        .run();

    let check_result = rx.recv().await.unwrap();
    agent.abort();
    let rcv_check = check_result
        .events
        .first()
        .expect("Expected to receive at least one check result");
    assert_eq!(
        rcv_check.status,
        isok_data::broker_rpc::CheckJobStatus::Reachable as i32
    );
}
