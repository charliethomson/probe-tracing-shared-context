use liberror::AnyError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error, valuable::Valuable)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "$type",
    content = "context"
)]
pub enum ListenerError {
    #[serde(rename = "dev.thmsn.sample.listener.error.unimplemented")]
    #[error("Unimplemented")]
    Unimplemented,
    #[serde(rename = "dev.thmsn.sample.listener.error.mq.invalid_config")]
    #[error("Invalid MQ Config: {0}")]
    InvalidMqConfig(AnyError),
    #[serde(rename = "dev.thmsn.sample.listener.error.mq.connection_failed")]
    #[error("Unable to connect to MQ: {0}")]
    UnableToConnectToMQ(AnyError),
    #[serde(rename = "dev.thmsn.sample.listener.error.mq.server")]
    #[error(transparent)]
    ServerError(#[from] libmq::server::MessageQueueServerError),
}
pub type ListenerResult<T> = Result<T, ListenerError>;
