use std::ops::Deref;

include!(concat!(env!("OUT_DIR"), "/isok.broker.rpc.rs"));

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