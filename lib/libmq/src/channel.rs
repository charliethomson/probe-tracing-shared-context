use derive_builder::Builder;

#[derive(Debug, Clone, Builder)]
pub struct ChannelConfiguration {
    #[builder(setter(into))]
    pub host: String,
    #[builder(setter(into))]
    pub stream_name: String,
    #[builder(default = 5552)]
    pub port: u16,
}
impl ChannelConfiguration {
    pub fn fmt(&self) -> String {
        format!("mq://{}:{}/{}", self.host, self.port, self.stream_name)
    }
}
