use std::{marker::PhantomData, time::Duration};

use futures::StreamExt;
use liberror::AnyError;
use serde::{Deserialize, Serialize};

use rabbitmq_stream_client::{
    error::StreamCreateError,
    types::{ByteCapacity, ResponseCode},
    Consumer, Environment, NoDedup, Producer,
};
use strum::Display;
use thiserror::Error;

use crate::{
    channel::ChannelConfiguration,
    message::{ManagerMessage, ManagerMessagePayload},
    meta::ManagerMeta,
    pack::{Packer, PackerError},
    payload::MessageQueuePayload,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, strum_macros::EnumDiscriminants, Error, valuable::Valuable,
)]
#[strum_discriminants(derive(Display))]
#[serde(
    tag = "$type",
    content = "reason",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MessageQueueServerError {
    #[error("Failed to create RabbitMQ environment: {0}")]
    #[serde(rename = "dev.thmsn.mq.server.create_environment")]
    CreateEnvironment(String),
    #[error("Failed to create RabbitMQ stream producer: {0}")]
    #[serde(rename = "dev.thmsn.mq.server.create_producer")]
    CreateProducer(AnyError),
    #[error("Failed to create RabbitMQ stream consumer: {0}")]
    #[serde(rename = "dev.thmsn.mq.server.create_consumer")]
    CreateConsumer(AnyError),
    #[error("Failed to send message: {0}")]
    #[serde(rename = "dev.thmsn.mq.server.send")]
    Send(AnyError),
    #[error("Failed to receive message: {0}")]
    #[serde(rename = "dev.thmsn.mq.server.receive")]
    Receive(AnyError),
    #[error("Failed to process message: {0}")]
    #[serde(rename = "dev.thmsn.mq.server.packer")]
    Packer(#[from] PackerError),
}
pub type MessageQueueServerResult<T> = Result<T, MessageQueueServerError>;

pub struct MessageQueueServer<
    TCall: MessageQueuePayload,
    TResponse: MessageQueuePayload,
    TPacker: Packer,
> {
    service_name: String,
    producer: Producer<NoDedup>,
    consumer: Consumer,
    _phantom_call: PhantomData<TCall>,
    _phantom_response: PhantomData<TResponse>,
    _phantom_packer: PhantomData<TPacker>,
}

impl<TCall: MessageQueuePayload, TResponse: MessageQueuePayload, TPacker: Packer>
    MessageQueueServer<TCall, TResponse, TPacker>
{
    #[tracing::instrument(name = "mq.server.new")]
    pub async fn new(
        service_name: String,
        mq: &ChannelConfiguration,
    ) -> MessageQueueServerResult<Self> {
        let environment = Environment::builder()
            .host(&mq.host)
            .port(mq.port)
            .build()
            .await
            .map_err(|e| MessageQueueServerError::CreateEnvironment(e.to_string()))?;

        // Ensure the stream exists
        let created = environment
            .stream_creator()
            .max_length(ByteCapacity::GB(1))
            .create(&mq.stream_name)
            .await;

        match created {
            Err(StreamCreateError::Create { status, .. }) => {
                // Stream exists, ignore
                match status {
                    ResponseCode::StreamAlreadyExists => {}
                    // general create error
                    _ => {
                        return Err(MessageQueueServerError::CreateEnvironment(format!(
                            "{:?}",
                            status
                        )))
                    }
                }
            }
            // No data
            Ok(()) => {}
            // general error
            _ => {
                return Err(MessageQueueServerError::CreateEnvironment(format!(
                    "{:?}",
                    created
                )))
            }
        }

        let producer = environment
            .producer()
            .build(&mq.stream_name)
            .await
            .map_err(|e| MessageQueueServerError::CreateProducer(e.into()))?;

        let consumer = environment
            .consumer()
            .build(&mq.stream_name)
            .await
            .map_err(|e| MessageQueueServerError::CreateConsumer(e.into()))?;

        Ok(Self {
            service_name,
            producer,
            consumer,
            _phantom_call: Default::default(),
            _phantom_response: Default::default(),
            _phantom_packer: Default::default(),
        })
    }

    fn pack_response(&self, response: TResponse) -> ManagerMessage<TCall, TResponse> {
        ManagerMessage::new_response(ManagerMeta::new(&self.service_name), response)
    }

    #[tracing::instrument(name = "mq.server.send", skip(self))]
    pub async fn send(&self, response: TResponse) -> MessageQueueServerResult<()> {
        let message = TPacker::pack(self.pack_response(response))?;
        self.producer
            // .send_with_confirm(message)
            .send(message, |_| async {})
            .await
            .map_err(|e| MessageQueueServerError::Send(e.into()))?;
        Ok(())
    }
    #[tracing::instrument(name = "mq.server.send_with_confirm", skip(self))]
    pub async fn send_with_confirm(&self, response: TResponse) -> MessageQueueServerResult<()> {
        let message = TPacker::pack(self.pack_response(response))?;
        self.producer
            .send_with_confirm(message)
            .await
            .map_err(|e| MessageQueueServerError::Send(e.into()))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> MessageQueueServerResult<Vec<TCall>> {
        let mut messages = vec![];
        loop {
            let result =
                tokio::time::timeout(Duration::from_micros(10), self.consumer.next()).await;
            let delivery = match result {
                Ok(Some(delivery)) => {
                    delivery.map_err(|e| MessageQueueServerError::Receive(e.into()))?
                }
                Ok(None) => break,
                Err(_elapsed) => break,
            };

            let message = delivery.message();
            let payload: ManagerMessage<TCall, TResponse> = TPacker::unpack(message)?;
            match payload.payload {
                ManagerMessagePayload::Call(manager_call) => {
                    let disc = manager_call.discriminant();
                    tracing::trace!("recv call {}: {}", delivery.offset(), disc);
                    messages.push(manager_call);
                }
                ManagerMessagePayload::Response(manager_response) => {
                    let disc = manager_response.discriminant();
                    tracing::trace!("ignore recv'd response {}: {}", delivery.offset(), disc);
                }
            }
        }
        Ok(messages)
    }
}
