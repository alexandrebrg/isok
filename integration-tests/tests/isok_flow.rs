use isok_agent::config::{Config as AgentConfig, ConfigCheckAdapter, ResultSenderAdapter};
use isok_agent::jobs::http::HttpJob;
use isok_agent::jobs::tcp::TcpJob;
use isok_agent::jobs::{Job, JobInnerConfig};
use isok_data::broker_rpc::CheckBatchRequest;
use pretty_assertions::assert_eq;
use prost::Message;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

static NEXT_PORT: AtomicUsize = AtomicUsize::new(24000);

struct AgentTestingRunner {
    config: AgentConfig,
}

impl AgentTestingRunner {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
        }
    }

    fn add_check(mut self, check: Job) -> Self {
        if let ConfigCheckAdapter::Static(adapter) = &mut self.config.check_config_adapter {
            adapter.checks.push(check);
        }
        self
    }

    fn use_socket_sender(mut self, path: PathBuf) -> Self {
        self.config.result_sender_adapter =
            ResultSenderAdapter::Socket(isok_agent::config::SocketConfig { path });
        self
    }

    fn create_path_socket_listener<T: Message + Default + 'static>(
        path: PathBuf,
    ) -> (UnboundedReceiver<T>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            let listener = UnixListener::bind(&path).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read message length
            let mut len_bytes = [0u8; 8];
            stream.read_exact(&mut len_bytes).await.unwrap();
            let len = u64::from_be_bytes(len_bytes) as usize;

            // Read message bytes
            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer).await.unwrap();

            // Decode the message
            let message = T::decode(&buffer[..]).unwrap();
            tx.send(message).unwrap();
        });
        (rx, handle)
    }

    fn create_tcp_socket_listener<T: Message + Default + 'static>(
        port: u16,
    ) -> (UnboundedReceiver<T>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
                .await
                .unwrap();
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = Vec::new();
                stream.read_to_end(&mut buffer).await.unwrap();
                let message = T::decode(&buffer[..]).unwrap();
                tx.send(message).unwrap();
            }
        });
        (rx, handle)
    }

    async fn start_threads(self) {
        tracing::info!("TESTS - Starting agent");
        isok_agent::run(self.config)
            .await
            .expect("Unable to run agent");
        tracing::info!("TESTS - Agent stopped");
    }

    fn run(self) -> JoinHandle<()> {
        tokio::spawn(self.start_threads())
    }
}

#[tokio::test]
async fn test_agent_feedback_ok_http() {
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
