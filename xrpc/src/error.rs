use std::fmt::Debug;

use actix_web::{HttpResponseBuilder, Responder, body::BoxBody, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Error, valuable::Valuable)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "$type",
    content = "context"
)]
pub enum ApiError {
    #[serde(rename = "dev.thmsn.sample.xrpc.error.send")]
    #[error("Failed to send message: {0}")]
    Send(libmq::client::MessageQueueClientError),
}

#[derive(Debug, Serialize)]
pub struct ApiResult<T: Debug + Serialize> {
    ok: bool,
    data: Option<T>,
    error: Option<ApiError>,
}
impl<T: Debug + Serialize> ApiResult<T> {
    pub fn new(ok: bool, data: Option<T>, error: Option<ApiError>) -> Self {
        Self { ok, data, error }
    }

    pub fn ok(data: T) -> Self {
        Self::new(true, Some(data), None)
    }

    pub fn err<E: Into<ApiError>>(error: E) -> Self {
        Self::new(true, None, Some(error.into()))
    }

    pub fn status_code(&self) -> StatusCode {
        let Some(error) = self.error.as_ref() else {
            return StatusCode::OK;
        };

        match error {
            ApiError::Send(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl<T: Debug + Serialize, E: Into<ApiError>> From<Result<T, E>> for ApiResult<T> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(data) => Self::ok(data),
            Err(error) => Self::err(error),
        }
    }
}
impl<T: Debug + Serialize> Responder for ApiResult<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut response = HttpResponseBuilder::new(self.status_code());

        response.json(&self)
    }
}
