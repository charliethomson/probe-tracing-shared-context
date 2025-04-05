use libmq::payload::MessageQueuePayload;
use libtran::Transaction;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

#[derive(EnumDiscriminants, Debug, Clone, Serialize, Deserialize)]
#[strum_discriminants(derive(strum::Display))]
pub enum ResponsePayload {
    #[serde(rename = "dev.thmsn.sample.response.result")]
    Result { result: f32 },
    #[serde(rename = "dev.thmsn.sample.response.too_big")]
    TooBig { lhs: f32, rhs: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub transaction: Transaction,
    pub payload: ResponsePayload,
}
impl Response {
    pub fn new(payload: ResponsePayload) -> Self {
        Self {
            transaction: Transaction::default(),
            payload,
        }
    }

    pub fn with_transaction(mut self, transaction: Transaction) -> Self {
        self.transaction = transaction;
        self
    }
}

impl MessageQueuePayload for Response {
    type Discriminant = ResponsePayloadDiscriminants;

    fn discriminant(&self) -> Self::Discriminant {
        Self::Discriminant::from(&self.payload)
    }
}
