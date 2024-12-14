use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use anyhow::Result;
use clap::ValueEnum;
use tracing::{instrument, trace, warn};

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    /// rust debug print.
    Debug,
    /// pretty printed json.
    Json,
    /// raw printed json.
    JsonR,
    /// yaml.
    Yaml,
}

#[instrument(err)]
pub fn wrap_command(command: &mut Command) -> Result<Output> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // Use log crate to allow verbosity flag to control wrapped command logs.
    if output.status.success() && !output.stdout.is_empty() {
        trace!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else if !output.stderr.is_empty() {
        warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(output)
}

#[instrument(err)]
pub fn get_directories(path: &Path) -> Result<Vec<PathBuf>> {
    Ok(fs::read_dir(path)?
        .filter_map(|dir| match dir {
            Ok(dir) => match dir.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        Some(dir.path())
                    } else {
                        None
                    }
                }
                Err(err) => {
                    eprintln!("An error occurred, skipping entry: {err}");
                    None
                }
            },
            Err(err) => {
                eprintln!("An error occurred, skipping entry: {err}");
                None
            }
        })
        .collect())
}
