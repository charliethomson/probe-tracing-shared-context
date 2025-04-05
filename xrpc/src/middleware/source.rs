use std::future::{Ready, ready};

use actix_web::FromRequest;

pub struct Source(String);
#[allow(unused)]
impl Source {
    pub fn into_inner(self) -> String {
        self.0
    }
    pub fn inner(&self) -> &str {
        &self.0
    }
}
impl FromRequest for Source {
    type Error = actix_web::Error;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let headers = req.headers();

        let header = headers
            .get("x-source")
            .map(|value| value.to_str())
            .transpose()
            .ok()
            .flatten()
            .unwrap_or("Unknown");

        let source = header.to_string();

        ready(Ok(Source(source)))
    }
}
