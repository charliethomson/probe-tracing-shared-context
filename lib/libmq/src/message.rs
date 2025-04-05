use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

use crate::meta::ManagerMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "TCall: DeserializeOwned, TResponse: DeserializeOwned"))]
pub enum ManagerMessagePayload<TCall, TResponse>
where
    TCall: Debug + Clone + Serialize + DeserializeOwned,
    TResponse: Debug + Clone + Serialize + DeserializeOwned,
{
    Call(TCall),
    Response(TResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "TCall: DeserializeOwned, TResponse: DeserializeOwned"))]
pub struct ManagerMessage<TCall, TResponse>
where
    TCall: Debug + Clone + Serialize + DeserializeOwned,
    TResponse: Debug + Clone + Serialize + DeserializeOwned,
{
    pub meta: ManagerMeta,
    pub payload: ManagerMessagePayload<TCall, TResponse>,
}
impl<
        TCall: Debug + Clone + Serialize + DeserializeOwned,
        TResponse: Debug + Clone + Serialize + DeserializeOwned,
    > ManagerMessage<TCall, TResponse>
{
    pub fn new(meta: ManagerMeta, payload: ManagerMessagePayload<TCall, TResponse>) -> Self {
        Self { meta, payload }
    }

    pub fn new_call(meta: ManagerMeta, call: TCall) -> Self {
        Self::new(meta, ManagerMessagePayload::Call(call))
    }
    pub fn new_response(meta: ManagerMeta, response: TResponse) -> Self {
        Self::new(meta, ManagerMessagePayload::Response(response))
    }
}
