use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{BatchConfig, RandomIdGenerator, Sampler, Tracer},
};
use tracing_core::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use super::{filter::get_global_filter, resource};

/// Amazing video on how this works: https://youtu.be/21rtHinFA40?si=vgARg2zxZ0ixC-yu
pub fn tracing_subscriber_init(log_filter: LevelFilter) -> TracerGuard {
    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(log_filter);

    let otel_layer = std::env::var("OTEL_COLLECTOR_URL")
        .map_or_else(
            |_| None,
            |url| Some(OpenTelemetryLayer::new(init_tracer(&url))),
        )
        .with_filter(get_global_filter());

    // TODO: per layer filter
    // - RUST_LOG for otel.
    // - clap verbosity flag for logger.

    tracing_subscriber::registry()
        // .with(get_global_filter())
        .with(log_layer)
        .with(otel_layer)
        .init();

    TracerGuard
}

// Construct Tracer for OpenTelemetryLayer
fn init_tracer(url: &str) -> Tracer {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                // Customize sampling strategy
                .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                    1.0,
                ))))
                // If export trace to AWS X-Ray, you can use XrayIdGenerator
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource::get_resource()),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(url),
        )
        .with_batch_config(BatchConfig::default())
        .install_batch(runtime::Tokio)
        .expect("opentelemetry tracer to configure correctly")
}

// Make sure we shutdown the tracer.
pub struct TracerGuard;

impl Drop for TracerGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
