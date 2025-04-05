
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    runtime::{self},
    trace::{RandomIdGenerator, Sampler, TracerProvider},
    Resource,
};
use opentelemetry_semantic_conventions::{
    attribute::{SERVICE_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub use opentelemetry::propagation::Extractor;
pub use opentelemetry::propagation::Injector;
pub use tracing_opentelemetry::OpenTelemetrySpanExt;

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource(service_name: &String) -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, service_name.to_string()),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new("deployment.environment.name", "develop"),
        ],
        SCHEMA_URL,
    )
}

fn init_tracer_provider(service_name: &String) -> TracerProvider {
    TracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_max_events_per_span(16)
        .with_resource(resource(service_name))
        .build()
}

pub struct LoggingGuard {
    pub _tracer_provider: TracerProvider,
}

#[derive(Default, Clone, Debug)]
pub enum OpenTelemetryEndpoint {
    Some(String),
    #[default]
    None,
}
impl<S: ToString> From<S> for OpenTelemetryEndpoint {
    fn from(value: S) -> Self {
        Self::Some(value.to_string())
    }
}
pub fn register_tracing_subscriber<S1: Into<OpenTelemetryEndpoint>, S2: ToString>(
    endpoint: S1,
    service_name: S2,
) -> LoggingGuard {
    let service_name_str = service_name.to_string();
    let res = resource(&service_name_str);

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::default());

    // Create a base subscriber stack
    let stack = tracing_subscriber::registry()
        .with(EnvFilter::from_env("SAMPLELOG_LEVEL"))
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format().pretty()),
        );

    let tracer_provider = if let OpenTelemetryEndpoint::Some(endpoint) = endpoint.into() {
        let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .unwrap();

        let provider = TracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_id_generator(RandomIdGenerator::default())
            .with_max_events_per_span(64)
            .with_max_attributes_per_span(16)
            .with_resource(res)
            .with_batch_exporter(otlp_exporter, runtime::Tokio)
            .build();

        let tracer = provider.tracer(service_name_str);

        stack.with(OpenTelemetryLayer::new(tracer)).init();

        provider
    } else {
        stack.init();

        init_tracer_provider(&service_name_str)
    };

    LoggingGuard {
        _tracer_provider: tracer_provider,
    }
}

pub fn force_cleanup(guard: LoggingGuard) {
    drop(guard._tracer_provider);
    global::shutdown_tracer_provider();
}

pub fn inject<I: Injector>(injector: &mut I) {
    let span = tracing::Span::current();
    let cx = span.context();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, injector);
    })
}

pub fn extract<E: Extractor>(extractor: &mut E) -> &mut E {
    let span = tracing::Span::current();
    let cx =
        opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(extractor));

    span.set_parent(cx);

    extractor
}
