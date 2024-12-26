use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobPrettyName {
    inner: String,
}

impl JobPrettyName {
    pub fn new(inner: String) -> Self {
        Self { inner }
    }
}

impl Deref for JobPrettyName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub mod broker_rpc {
    tonic::include_proto!("isok.broker.rpc");
    pub type BrokerGrpcClient = broker_client::BrokerClient<tonic::transport::Channel>;
}

