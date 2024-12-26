use tokio::sync::RwLock;
use isok_data::broker_rpc::CheckResult;

enum Broker {
    Grpc,
    Dev(DevBroker),
}

#[async_trait::async_trait]
impl SendEvents for Broker {
    async fn send_events(&self, event: CheckResult) -> crate::Result<()> {
        match self {
            Broker::Grpc => Ok(()),
            Broker::Dev(broker) => broker.send_events(event).await,
        }
    }
}

#[async_trait::async_trait]
trait SendEvents {
    async fn send_events(&self, event: CheckResult) -> crate::Result<()>;
}

struct DevBroker {
    events: RwLock<Vec<CheckResult>>,
}

#[async_trait::async_trait]
impl SendEvents for DevBroker {
    async fn send_events(&self, event: CheckResult) -> crate::Result<()> {
        self.events.write().await.push(event);

        Ok(())
    }
}
