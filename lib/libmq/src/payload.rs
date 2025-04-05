use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait MessageQueuePayload: Debug + Clone + Serialize + DeserializeOwned {
    type Discriminant: std::fmt::Display;

    fn discriminant(&self) -> Self::Discriminant;
}
