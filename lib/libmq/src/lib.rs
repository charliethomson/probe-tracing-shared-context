pub mod channel;
pub mod client;
pub mod message;
pub mod meta;
pub mod pack;
pub mod payload;
pub mod server;

#[macro_export]
macro_rules! nt_channel {
    ($clientname:ident,$servname:ident,$call:ty,$resp:ty,$packer:ty) => {
        pub struct $clientname(libmq::client::MessageQueueClient<$call, $resp, $packer>);
        impl $clientname {
            pub async fn new(
                client_name: String,
                mq_config: &libmq::channel::ChannelConfiguration,
            ) -> libmq::client::MessageQueueClientResult<Self> {
                Ok(Self(
                    libmq::client::MessageQueueClient::new(client_name, mq_config).await?,
                ))
            }
        }
        impl std::ops::Deref for $clientname {
            type Target = libmq::client::MessageQueueClient<$call, $resp, $packer>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $clientname {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        pub struct $servname(libmq::server::MessageQueueServer<$call, $resp, $packer>);
        impl $servname {
            pub async fn new(
                service_name: String,
                mq_config: &libmq::channel::ChannelConfiguration,
            ) -> libmq::server::MessageQueueServerResult<Self> {
                Ok(Self(
                    libmq::server::MessageQueueServer::new(service_name, mq_config).await?,
                ))
            }
        }
        impl std::ops::Deref for $servname {
            type Target = libmq::server::MessageQueueServer<$call, $resp, $packer>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $servname {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}
