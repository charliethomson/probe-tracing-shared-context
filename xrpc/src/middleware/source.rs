use std::future::{Ready, ready};

use actix_web::{
    Error, FromRequest, HttpMessage, HttpResponse, HttpResponseBuilder,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;

use crate::error::ApiResult;

struct SourceExtension(Option<String>);

pub struct Source(String);
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
        let extensions = req.extensions();

        let Some(extension) = extensions.get::<SourceExtension>() else {
            return ready(Err(actix_web::error::ErrorInternalServerError(
                "Missing source extension",
            )));
        };

        let source = extension
            .0
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
            .to_string();

        ready(Ok(Source(source)))
    }
}

pub struct SourceMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for SourceMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SourceMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SourceMiddleware { service }))
    }
}

pub struct SourceMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SourceMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();

        let _guard = tracing::info_span!("middleware.source.extract").entered();

        tracing::info!("Extracting soruce header");

        let source = headers
            .get("x-source")
            .map(|value| value.to_str())
            .transpose()
            .ok()
            .flatten()
            .map(|source| source.to_string());

        if let Some(source) = source.as_ref() {
            tracing::info!("Source header found: {}", source);
        } else {
            tracing::warn!("Source header not found");
        }

        tracing::debug!("Adding source extension");

        let extension = SourceExtension(source);
        req.extensions_mut().insert(extension);

        tracing::info!("Source extension added");

        drop(_guard);

        let fut = self.service.call(req);

        Box::pin(async move { fut.await })
    }
}
