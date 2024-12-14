use anyhow::Result;
use axl_lib::config::config_file::OnError;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use inquire::Text;
use std::{env, process::exit, time::Duration};
use tracing::{error, info_span};
use tracing_log::AsTrace;
use uuid::Uuid;

use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub mod cli;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    // set layer for log subscriber
    tracing_subscriber::registry()
        .with(cli.args.verbosity.log_level_filter().as_trace())
        .with(fmt::layer().pretty())
        .init();

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
                            std::thread::sleep(Duration::from_millis(500));
                        }
                        OnError::LongDelay => {
                            std::thread::sleep(Duration::from_millis(5000));
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

    Ok(())
}
