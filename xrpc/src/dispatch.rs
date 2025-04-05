use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Data, Json},
};
use libtran::Transaction;
use tracing::{Level, instrument};

use crate::{
    error::ApiResult,
    methods::{add, div, mul, sub},
    middleware::source::Source,
    state::AppState,
};

#[post("/add")]
#[instrument(skip_all)]
pub async fn add_dispatch(
    source: Source,
    data: Data<AppState>,
    payload: Json<add::AddEndpointPayload>,
) -> impl Responder {
    let mut tran = Transaction::default()
        .with_source(source.inner())
        .with_intent("dev.thmsn.operation.add");
    tran.inject();

    ApiResult::from(add::add_endpoint(tran, &data.client, payload.into_inner()).await)
}

#[post("/sub")]
#[instrument(skip_all)]
pub async fn sub_dispatch(
    source: Source,
    data: Data<AppState>,
    payload: Json<sub::SubEndpointPayload>,
) -> impl Responder {
    let mut tran = Transaction::default()
        .with_source(source.inner())
        .with_intent("dev.thmsn.operation.sub");
    tran.inject();

    ApiResult::from(sub::sub_endpoint(tran, &data.client, payload.into_inner()).await)
}

#[post("/mul")]
#[instrument(skip_all)]
pub async fn mul_dispatch(
    source: Source,
    data: Data<AppState>,
    payload: Json<mul::MulEndpointPayload>,
) -> impl Responder {
    let mut tran = Transaction::default()
        .with_source(source.inner())
        .with_intent("dev.thmsn.operation.mul");
    tran.inject();

    ApiResult::from(mul::mul_endpoint(tran, &data.client, payload.into_inner()).await)
}

#[post("/div")]
#[instrument(skip_all)]
pub async fn div_dispatch(
    source: Source,
    data: Data<AppState>,
    payload: Json<div::DivEndpointPayload>,
) -> impl Responder {
    let mut tran = Transaction::default()
        .with_source(source.inner())
        .with_intent("dev.thmsn.operation.div");
    tran.inject();

    ApiResult::from(div::div_endpoint(tran, &data.client, payload.into_inner()).await)
}

#[get("/health")]
#[instrument(skip_all, level = Level::TRACE)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json("Healthy")
}

pub fn configure(conf: &mut web::ServiceConfig) {
    let main_scope = web::scope("/dev.thmsn.sample")
        .service(add_dispatch)
        .service(sub_dispatch)
        .service(mul_dispatch)
        .service(div_dispatch);

    conf.service(main_scope).service(health);
}
