use libmq::payload::MessageQueuePayload;
use libtran::Transaction;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

#[derive(EnumDiscriminants, Debug, Clone, Serialize, Deserialize)]
#[strum_discriminants(derive(strum::Display))]
pub enum CallPayload {
    #[serde(rename = "dev.thmsn.sample.call.add")]
    Add { lhs: f32, rhs: f32 },
    #[serde(rename = "dev.thmsn.sample.call.sub")]
    Sub { lhs: f32, rhs: f32 },
    #[serde(rename = "dev.thmsn.sample.call.mul")]
    Mul { lhs: f32, rhs: f32 },
    #[serde(rename = "dev.thmsn.sample.call.div")]
    Div { lhs: f32, rhs: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub transaction: Transaction,
    pub payload: CallPayload,
}

impl MessageQueuePayload for Call {
    type Discriminant = CallPayloadDiscriminants;

    fn discriminant(&self) -> Self::Discriminant {
        Self::Discriminant::from(&self.payload)
    }
}
