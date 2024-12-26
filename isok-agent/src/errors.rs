use crate::batch_sender::BatchSenderError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to create job registry")]
    UnableToCreateJobRegistry(#[from] figment::Error),
    #[error("Config path provided is invalid")]
    InvalidConfigPath,
    #[error("Unable to create batch sender")]
    UnableToCreateBatchSender(#[from] BatchSenderError),
}
