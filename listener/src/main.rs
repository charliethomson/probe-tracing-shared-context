use app::App;
use clap::Parser;
use error::ListenerResult;
use tokio_util::sync::CancellationToken;

mod app;
mod error;

pub const SERVICE_NAME: &str = "dev.thmsn.sample.listener";

#[derive(Parser)]
pub struct Args {
    #[arg(long, env)]
    pub otel_endpoint: String,
    #[arg(long, env)]
    pub mq_host: String,
    #[arg(long, env)]
    pub mq_port: u16,
    #[arg(long, env)]
    pub mq_stream: String,
}

#[tokio::main]
async fn main() -> ListenerResult<()> {
    let args = Args::parse();

    let _logging_guard = liblog::register_tracing_subscriber(&args.otel_endpoint, SERVICE_NAME);

    let cancellation_token = CancellationToken::new();

    {
        let cancellation_token = cancellation_token.clone();
        tokio::spawn(async move {
            libsignal::wait_for_signal().await;
            cancellation_token.cancel();
        });
    }

    let app = App::new(cancellation_token.child_token(), args).await?;

    app.run().await?;

    liblog::force_cleanup(_logging_guard);

    Ok(())
}
