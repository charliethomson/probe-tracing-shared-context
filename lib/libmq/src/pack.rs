use liberror::AnyError;
use rabbitmq_stream_client::types::Message;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize, valuable::Valuable)]
#[serde(tag = "$type", content = "reason")]
pub enum PackerValidateError {
    #[error("No properties are present")]
    #[serde(rename = "dev.thmsn.mq.packer.validate.no_properties")]
    NoProperties,
    #[error("Property named \"{property_name}\" not found")]
    #[serde(rename = "dev.thmsn.mq.packer.validate.missing_property")]
    MissingProperty { property_name: String },
    #[error(
        "Property named \"{property_name}\" expected \"{expected_value}\" got \"{actual_value}\""
    )]
    #[serde(rename = "dev.thmsn.mq.packer.validate.incorrect_property_value")]
    IncorrectPropertyValue {
        property_name: String,
        expected_value: String,
        actual_value: String,
    },
}

#[derive(Debug, Clone, Error, Serialize, Deserialize, valuable::Valuable)]
#[serde(tag = "$type", content = "reason")]
pub enum PackerError {
    #[error("Failed to serialize message: {0}")]
    #[serde(rename = "dev.thmsn.mq.packer.serialize")]
    Serialize(AnyError),
    #[error("Failed to deserialize message: {0}")]
    #[serde(rename = "dev.thmsn.mq.packer.deserialize")]
    Deserialize(AnyError),

    #[error("Failed to validate message: {0}")]
    #[serde(rename = "dev.thmsn.mq.packer.validation")]
    Validation(#[from] PackerValidateError),

    #[error("Unable to parse message, no body present")]
    #[serde(rename = "dev.thmsn.mq.packer.missing_body")]
    MissingBody,
}
pub type PackerResult<T> = Result<T, PackerError>;

pub trait Packer: std::fmt::Debug {
    const CONTENT_TYPE: &'static str;

    fn ser<Payload: Serialize>(payload: &Payload) -> PackerResult<Vec<u8>>;
    fn de<Payload: DeserializeOwned>(bytes: &[u8]) -> PackerResult<Payload>;

    fn pack<Payload: Serialize>(payload: Payload) -> PackerResult<Message> {
        let bytes = Self::ser(&payload)?;
        Ok(Message::builder()
            .body(bytes)
            .properties()
            .content_type(Self::CONTENT_TYPE)
            .message_builder()
            .build())
    }

    fn unpack<Payload: DeserializeOwned>(message: &Message) -> PackerResult<Payload> {
        Self::validate(message)?;

        let body = message.data().ok_or(PackerError::MissingBody)?;
        let payload = Self::de(body)?;
        Ok(payload)
    }

    fn validate(message: &Message) -> PackerResult<()> {
        let properties = message
            .properties()
            .ok_or(PackerValidateError::NoProperties)?;

        let content_type =
            properties
                .content_type
                .as_ref()
                .ok_or(PackerValidateError::MissingProperty {
                    property_name: "content_type".into(),
                })?;
        if Self::CONTENT_TYPE != content_type.as_str() {
            return Err(PackerValidateError::IncorrectPropertyValue {
                property_name: "content_type".into(),
                expected_value: Self::CONTENT_TYPE.to_string(),
                actual_value: content_type.to_string(),
            }
            .into());
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MessagePackPacker;

impl Packer for MessagePackPacker {
    const CONTENT_TYPE: &'static str = "application/rmp_serde";

    fn ser<Payload: Serialize>(payload: &Payload) -> PackerResult<Vec<u8>> {
        rmp_serde::to_vec_named(payload).map_err(|e| PackerError::Serialize(e.into()))
    }

    fn de<Payload: DeserializeOwned>(bytes: &[u8]) -> PackerResult<Payload> {
        rmp_serde::from_slice(bytes).map_err(|e| PackerError::Deserialize(e.into()))
    }
}

#[derive(Debug)]
pub struct JsonPacker;

impl Packer for JsonPacker {
    const CONTENT_TYPE: &'static str = "application/json";

    fn ser<Payload: Serialize>(payload: &Payload) -> PackerResult<Vec<u8>> {
        serde_json::to_vec(payload).map_err(|e| PackerError::Serialize(e.into()))
    }

    fn de<Payload: DeserializeOwned>(bytes: &[u8]) -> PackerResult<Payload> {
        serde_json::from_slice(bytes).map_err(|e| PackerError::Deserialize(e.into()))
    }
}
