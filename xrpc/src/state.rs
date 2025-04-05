use std::sync::Arc;

use libshared::mq::SampleClient;

pub struct AppState {
    pub client: Arc<SampleClient>,
}
