mod dispatch;
mod error;
mod methods;
mod middleware;
mod state;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web::Data};
use clap::Parser;
use dispatch::configure;
use liblog::register_tracing_subscriber;
use libshared::mq::SampleClient;
use middleware::source::SourceMiddlewareFactory;
use state::AppState;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, env)]
    otel_endpoint: String,
    #[arg(long, env)]
    pub mq_host: String,
    #[arg(long, env)]
    pub mq_port: u16,
    #[arg(long, env)]
    pub mq_stream: String,
}

const SERVICE_NAME: &str = "dev.thmsn.sample.xrpc";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let guard = register_tracing_subscriber(args.otel_endpoint, SERVICE_NAME);

    let client = {
        let _guard = tracing::info_span!("app.init").entered();
        let conf = libmq::channel::ChannelConfigurationBuilder::default()
            .host(&args.mq_host)
            .port(args.mq_port)
            .stream_name(&args.mq_stream)
            .build()?;
        SampleClient::new(SERVICE_NAME.to_string(), &conf).await?
    };

    let client = Arc::new(client);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                client: client.clone(),
            }))
            .wrap(SourceMiddlewareFactory)
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .wrap(TracingLogger::default())
            .configure(configure)
    })
    .bind(("0.0.0.0", args.port))?;

    tracing::warn!("Starting server @ 0.0.0.0:{}", args.port);

    let result = server.run().await;

    match result.as_ref() {
        Ok(_) => tracing::info!("Shutting down normally."),
        Err(e) => tracing::error!("Fatal error occurred: {}", e),
    }
    drop(guard);

    result?;

    Ok(())
}
