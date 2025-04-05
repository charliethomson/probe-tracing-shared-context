use libshared::mq::{
    SampleServer,
    call::{Call, CallPayload},
    response::{Response, ResponsePayload},
};
use tokio_util::sync::CancellationToken;

use crate::{
    Args, SERVICE_NAME,
    error::{ListenerError, ListenerResult},
};

pub struct App {
    cancellation_token: CancellationToken,
    server: SampleServer,
}

impl App {
    pub async fn new(cancellation_token: CancellationToken, args: Args) -> ListenerResult<Self> {
        let server = {
            let conf = libmq::channel::ChannelConfigurationBuilder::default()
                .host(&args.mq_host)
                .port(args.mq_port)
                .stream_name(&args.mq_stream)
                .build()
                .map_err(|e| ListenerError::InvalidMqConfig(e.into()))?;
            SampleServer::new(SERVICE_NAME.to_string(), &conf)
                .await
                .map_err(|e| ListenerError::UnableToConnectToMQ(e.into()))?
        };

        Ok(Self {
            cancellation_token,
            server,
        })
    }

    pub async fn run(mut self) -> ListenerResult<()> {
        while !self.cancellation_token.is_cancelled() {
            self.tick().await?;
        }

        Ok(())
    }
}

impl App {
    async fn tick(&mut self) -> ListenerResult<()> {
        let deliveries = self.server.recv().await?;

        for delivery in deliveries {
            match delivery.payload {
                CallPayload::Add { lhs, rhs } => self.process_add(delivery, lhs, rhs).await?,
                CallPayload::Sub { lhs, rhs } => self.process_sub(delivery, lhs, rhs).await?,
                CallPayload::Mul { lhs, rhs } => self.process_mul(delivery, lhs, rhs).await?,
                CallPayload::Div { lhs, rhs } => self.process_div(delivery, lhs, rhs).await?,
            };
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, delivery))]
    async fn process_add(&mut self, mut delivery: Call, lhs: f32, rhs: f32) -> ListenerResult<()> {
        delivery.transaction.extract();

        tracing::info!(lhs = lhs, rhs = rhs, "Processing add operation");

        let result = lhs + rhs;
        let response: ResponsePayload = if result > 100f32 {
            tracing::error!(result = result, "Result is too big!");
            ResponsePayload::TooBig { lhs, rhs }
        } else {
            ResponsePayload::Result { result }
        };
        let response = Response::new(response).with_transaction(delivery.transaction);
        self.server.send(response).await?;

        tracing::info!(result = result, "add operation completed successfully");
        Ok(())
    }

    #[tracing::instrument(skip(self, delivery))]
    async fn process_sub(&mut self, mut delivery: Call, lhs: f32, rhs: f32) -> ListenerResult<()> {
        delivery.transaction.extract();

        tracing::info!(lhs = lhs, rhs = rhs, "Processing sub operation");

        let result = lhs - rhs;
        let response: ResponsePayload = if result > 100f32 {
            tracing::error!(result = result, "Result is too big!");
            ResponsePayload::TooBig { lhs, rhs }
        } else {
            ResponsePayload::Result { result }
        };
        let response = Response::new(response).with_transaction(delivery.transaction);
        self.server.send(response).await?;

        tracing::info!(result = result, "sub operation completed successfully");
        Ok(())
    }

    #[tracing::instrument(skip(self, delivery))]
    async fn process_mul(&mut self, mut delivery: Call, lhs: f32, rhs: f32) -> ListenerResult<()> {
        delivery.transaction.extract();

        tracing::info!(lhs = lhs, rhs = rhs, "Processing mul operation");

        let result = lhs * rhs;
        let response: ResponsePayload = if result > 100f32 {
            tracing::error!(result = result, "Result is too big!");
            ResponsePayload::TooBig { lhs, rhs }
        } else {
            ResponsePayload::Result { result }
        };
        let response = Response::new(response).with_transaction(delivery.transaction);
        self.server.send(response).await?;

        tracing::info!(result = result, "mul operation completed successfully");
        Ok(())
    }

    #[tracing::instrument(skip(self, delivery))]
    async fn process_div(&mut self, mut delivery: Call, lhs: f32, rhs: f32) -> ListenerResult<()> {
        delivery.transaction.extract();

        tracing::info!(lhs = lhs, rhs = rhs, "Processing div operation");

        let result = lhs / rhs;
        let response: ResponsePayload = if result > 100f32 {
            tracing::error!(result = result, "Result is too big!");
            ResponsePayload::TooBig { lhs, rhs }
        } else {
            ResponsePayload::Result { result }
        };
        let response = Response::new(response).with_transaction(delivery.transaction);
        self.server.send(response).await?;

        tracing::info!(result = result, "div operation completed successfully");
        Ok(())
    }
}
