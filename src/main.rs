use anyhow::Result;
use axl_lib::config::config_file::OnError;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use inquire::Text;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use std::{env, process::exit, time::Duration};
use tokio::time::sleep;
use tracing::{error, info_span, metadata::LevelFilter};
use tracing_log::AsTrace;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};
use uuid::Uuid;

pub mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    configure_tracing(cli.args.verbosity.log_level_filter().as_trace())?;

    // So the span and guard are dropped before shutting down tracer provider.
    {
        // Create a uuid that can be provided to the user to more effectively search for the command trace.
        let trace_uuid = Uuid::new_v4();
        let root_span = info_span!(
            "main",
            run.uuid = trace_uuid.to_string(),
            executable.path = env::current_exe()
                .expect("binary execution should have a current executable")
                .to_string_lossy()
                .to_string(),
            executable.version = env!("CARGO_PKG_VERSION"),
        );
        let _guard = root_span.enter();

        // Somehow need to merge the cli arguments with the config file to allow for overriding values
        // with flags for testing.
        match cli.init() {
            Ok(cli) => match cli.handle_command() {
                Ok(_) => {}
                Err(err) => {
                    let msg = format!("[CMD ERROR]: {err:?}");
                    error!(run.uuid = trace_uuid.to_string(), msg,);
                    eprintln!("{}", msg.red().bold());
                    match cli.args.on_error {
                        OnError::None => {}
                        OnError::Pause => {
                            Text::new("Press ENTER to continue...").prompt()?;
                        }
                        OnError::ShortDelay => {
                            sleep(Duration::from_millis(500)).await;
                        }
                        OnError::LongDelay => {
                            sleep(Duration::from_millis(5000)).await;
                        }
                    }
                    exit(1)
                }
            },
            Err(err) => {
                let msg = format!("[INIT ERROR]: {err:?}");
                error!(run.uuid = trace_uuid.to_string(), msg,);
                eprintln!("{}", msg.red().bold());
                exit(1)
            }
        }
    }

    // This is needed to export all remaining spans before exiting.
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

/// Amazing video on how this works: https://youtu.be/21rtHinFA40?si=vgARg2zxZ0ixC-yu
fn configure_tracing(log_filter: LevelFilter) -> Result<()> {
    tracing_subscriber::registry()
        .with(
            // set layer for log subscriber
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_filter(log_filter),
        )
        .with(std::env::var("OTEL_COLLECTOR_URL").map_or_else(
            |_| None,
            |url| {
                Some(
                    tracing_opentelemetry::layer()
                        .with_tracer(
                            opentelemetry_otlp::new_pipeline()
                                .tracing()
                                .with_exporter(
                                    opentelemetry_otlp::new_exporter()
                                        .tonic()
                                        .with_endpoint(url),
                                )
                                .with_trace_config(
                                    opentelemetry_sdk::trace::config().with_resource(
                                        Resource::new(vec![KeyValue::new(
                                        opentelemetry_semantic_conventions::resource::SERVICE_NAME
                                            .to_string(),
                                        env!("CARGO_PKG_NAME"),
                                    )]),
                                    ),
                                )
                                .install_batch(opentelemetry_sdk::runtime::Tokio)
                                .expect("Failed creating the tracer!"),
                        )
                        .with_filter(
                            // If no `RUST_LOG` is provided use info.
                            tracing_subscriber::EnvFilter::try_from_default_env()
                                .unwrap_or_else(|_| "info".into()),
                        ),
                )
            },
        ))
        .init();

    Ok(())
}
