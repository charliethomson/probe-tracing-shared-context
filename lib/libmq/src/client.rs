use std::{marker::PhantomData, time::Duration};

use futures::StreamExt;
use liberror::AnyError;
use serde::{Deserialize, Serialize};
use tap::TapFallible;

use rabbitmq_stream_client::{
    error::StreamCreateError,
    types::{ByteCapacity, ResponseCode},
    Consumer, Environment, NoDedup, Producer,
};
use thiserror::Error;

use crate::{
    channel::ChannelConfiguration,
    message::{ManagerMessage, ManagerMessagePayload},
    meta::ManagerMeta,
    pack::{Packer, PackerError},
    payload::MessageQueuePayload,
};

#[derive(Debug, Clone, Error, Serialize, Deserialize, valuable::Valuable)]
#[serde(tag = "$type", content = "reason")]
pub enum MessageQueueClientError {
    #[error("Failed to create RabbitMQ environment: {0}")]
    #[serde(rename = "dev.thmsn.mq.client.create_environment")]
    CreateEnvironment(String),
    #[error("Failed to create RabbitMQ stream producer: {0}")]
    #[serde(rename = "dev.thmsn.mq.client.create_producer")]
    CreateProducer(AnyError),
    #[error("Failed to create RabbitMQ stream consumer: {0}")]
    #[serde(rename = "dev.thmsn.mq.client.create_consumer")]
    CreateConsumer(AnyError),
    #[error("Failed to send message: {0}")]
    #[serde(rename = "dev.thmsn.mq.client.send")]
    Send(AnyError),
    #[error("Failed to receive message: {0}")]
    #[serde(rename = "dev.thmsn.mq.client.receive")]
    Receive(AnyError),
    #[error("Failed to process message: {0}")]
    #[serde(rename = "dev.thmsn.mq.client.packer")]
    Packer(#[from] PackerError),
}
pub type MessageQueueClientResult<T> = Result<T, MessageQueueClientError>;

pub struct MessageQueueClient<
    TCall: MessageQueuePayload,
    TResponse: MessageQueuePayload,
    TPacker: Packer,
> {
    id: String,
    producer: Producer<NoDedup>,
    consumer: Consumer,
    _phantom_call: PhantomData<TCall>,
    _phantom_response: PhantomData<TResponse>,
    _phantom_packer: PhantomData<TPacker>,
}
impl<TCall: MessageQueuePayload, TResponse: MessageQueuePayload, TPacker: Packer>
    MessageQueueClient<TCall, TResponse, TPacker>
{
    #[tracing::instrument(name = "mq.client.new")]
    pub async fn new(
        client_name: String,
        mq_config: &ChannelConfiguration,
    ) -> MessageQueueClientResult<Self> {
        let environment = Environment::builder()
            .host(&mq_config.host)
            .port(mq_config.port)
            .build()
            .await
            .map_err(|e| MessageQueueClientError::CreateEnvironment(e.to_string()))?;

        tracing::info!("Environment created");

        // Ensure the stream exists
        let created = environment
            .stream_creator()
            .max_length(ByteCapacity::GB(1))
            .create(&mq_config.stream_name)
            .await;

        match created.as_ref() {
            Err(StreamCreateError::Create { status, .. }) => {
                // Stream exists, ignore
                match status {
                    ResponseCode::StreamAlreadyExists => {}
                    // general create error
                    e => {
                        tracing::error!("Failed to create stream (1): {e:?}");
                        return Err(MessageQueueClientError::CreateEnvironment(format!(
                            "{:?}",
                            status
                        )));
                    }
                }
            }
            // No data
            Ok(()) => {}
            // general error
            Err(e) => {
                tracing::error!(error = e.to_string(), "Failed to create stream (2): {e}");
                return Err(MessageQueueClientError::CreateEnvironment(format!(
                    "{:?}",
                    created
                )));
            }
        }

        let producer = environment
            .producer()
            .build(&mq_config.stream_name)
            .await
            .tap_err(|e| tracing::error!("{e:?}"))
            .map_err(|e| MessageQueueClientError::CreateProducer(e.into()))?;
        tracing::info!("Producer created");

        let consumer = environment
            .consumer()
            .build(&mq_config.stream_name)
            .await
            .map_err(|e| MessageQueueClientError::CreateConsumer(e.into()))?;
        tracing::info!("Consumer created");

        Ok(Self {
            id: client_name,
            producer,
            consumer,
            _phantom_call: Default::default(),
            _phantom_response: Default::default(),
            _phantom_packer: Default::default(),
        })
    }

    fn pack_call(&self, call: TCall) -> ManagerMessage<TCall, TResponse> {
        ManagerMessage::new_call(ManagerMeta::new(&self.id), call)
    }

    #[tracing::instrument(name = "mq.client.send", skip(self))]
    pub async fn send(&self, call: TCall) -> MessageQueueClientResult<()> {
        let message = TPacker::pack(self.pack_call(call))?;
        self.producer
            .send(message, |_| async {})
            .await
            .map_err(|e| MessageQueueClientError::Send(e.into()))?;
        Ok(())
    }

    #[tracing::instrument(name = "mq.client.send_with_confirm", skip(self))]
    pub async fn send_with_confirm(&self, call: TCall) -> MessageQueueClientResult<()> {
        let message = TPacker::pack(self.pack_call(call))?;
        self.producer
            .send_with_confirm(message)
            .await
            .map_err(|e| MessageQueueClientError::Send(e.into()))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> MessageQueueClientResult<Vec<TResponse>> {
        let mut messages = vec![];
        loop {
            let result =
                tokio::time::timeout(Duration::from_micros(10), self.consumer.next()).await;
            let delivery = match result {
                Ok(Some(delivery)) => {
                    delivery.map_err(|e| MessageQueueClientError::Receive(e.into()))?
                }
                Ok(None) => break,
                Err(_elapsed) => break,
            };

            let message = delivery.message();
            let payload: ManagerMessage<TCall, TResponse> = TPacker::unpack(message)?;
            match payload.payload {
                ManagerMessagePayload::Call(manager_call) => {
                    let disc = manager_call.discriminant();
                    tracing::trace!("ignore recv'd call {}: {}", delivery.offset(), disc);
                }
                ManagerMessagePayload::Response(manager_response) => {
                    let disc = manager_response.discriminant();
                    tracing::trace!("recv response {}: {}", delivery.offset(), disc);
                    messages.push(manager_response);
                }
            }
        }
        Ok(messages)
    }
}
