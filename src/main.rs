use anyhow::Result;
use axl_lib::config::config_file::OnError;
use axl_lib::tracing::otel::tracing_subscriber_init;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use inquire::Text;
use std::{process::exit, time::Duration};
use tokio::time::sleep;
use tracing_log::AsTrace;
use uuid::Uuid;

pub mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let mut exit_code = 0;

    // So the tracer, and span guards are dropped before shutting down tracer provider.
    {
        let _tracer_guard =
            tracing_subscriber_init(cli.args.verbosity.log_level_filter().as_trace());
        // Create a uuid that can be provided to the user to more effectively search for the command trace.
        let trace_uuid = Uuid::new_v4();
        let root_span = tracing::info_span!(
            "main",
            run.uuid = trace_uuid.to_string(),
            executable.path = std::env::current_exe()
                .expect("binary execution should have a current executable")
                .to_string_lossy()
                .to_string(),
            executable.version = env!("CARGO_PKG_VERSION"),
        );
        let _root_span_guard = root_span.enter();

        // Somehow need to merge the cli arguments with the config file to allow for overriding values
        // with flags for testing.
        match cli.init() {
            Ok(cli) => match cli.handle_command() {
                Ok(_) => {}
                Err(err) => {
                    let msg = format!("[CMD ERROR]: {err:?}");
                    tracing::error!(run.uuid = trace_uuid.to_string(), msg,);
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
                    exit_code = 1;
                }
            },
            Err(err) => {
                let msg = format!("[INIT ERROR]: {err:?}");
                tracing::error!(run.uuid = trace_uuid.to_string(), msg,);
                eprintln!("{}", msg.red().bold());
                exit_code = 1;
            }
        }
    }

    exit(exit_code);
}
