use libshared::mq::{SampleClient, call::Call};
use libtran::Transaction;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Serialize, Deserialize, Debug, Clone, valuable::Valuable)]
pub struct MulEndpointPayload {
    pub lhs: f32,
    pub rhs: f32,
}

#[tracing::instrument(skip(client))]
pub async fn mul_endpoint(
    transaction: Transaction,
    client: &SampleClient,
    payload: MulEndpointPayload,
) -> Result<(), ApiError> {
    let call = Call {
        transaction,
        payload: libshared::mq::call::CallPayload::Mul {
            lhs: payload.lhs,
            rhs: payload.rhs,
        },
    };
    client.send(call).await.map_err(|e| ApiError::Send(e))
}
