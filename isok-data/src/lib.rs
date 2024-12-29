use std::ops::Deref;
use ulid::Ulid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct JobId(Ulid);

impl JobId {
    pub fn generate() -> Self {
        Self(Ulid::new())
    }
}

impl From<Ulid> for JobId {
    fn from(value: Ulid) -> Self {
        Self(value)
    }
}

impl Deref for JobId {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
