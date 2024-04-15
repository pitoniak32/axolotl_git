use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use anyhow::Result;
use colored::Colorize;
use tracing::{info, instrument, warn};

use crate::{error::AxlError, fzf::FzfCmd, project::subcommand::OutputFormat};

#[instrument(err)]
pub fn wrap_command(command: &mut Command) -> Result<Output> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // Use log crate to allow verbosity flag to control wrapped command logs.
    if output.status.success() && !output.stdout.is_empty() {
        info!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else if !output.stderr.is_empty() {
        warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(output)
}

#[instrument(err)]
pub fn fzf_get_sessions(session_names: Vec<String>) -> Result<Vec<String>> {
    if session_names.is_empty() {
        eprintln!("\n{}\n", "No sessions found to choose from.".blue().bold());
        Err(AxlError::NoSessionsFound)?
    }

    Ok(FzfCmd::new()
        .args(vec!["--phony", "--multi"])
        .find_vec(session_names)?
        .trim_end()
        .split('\n')
        .map(|s| s.to_string())
        .collect())
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

pub fn formatted_print_iter<T>(output: &OutputFormat, value: impl Iterator<Item = T>) -> Result<()>
where
    T: std::fmt::Debug + serde::Serialize,
{
    let vecd = value.collect::<Vec<_>>();
    match output {
        OutputFormat::Debug => {
            println!("{:#?}", vecd);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&vecd)?)
        }
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&vecd)?),
        OutputFormat::JsonR => {
            println!("{}", serde_json::to_string(&vecd)?)
        }
    }
    Ok(())
}
