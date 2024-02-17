use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use anyhow::Result;
use colored::Colorize;
use tracing::{info, warn};

use crate::fzf::FzfCmd;

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

pub fn fzf_get_sessions(session_names: Vec<String>) -> Result<Vec<String>> {
    if session_names.is_empty() {
        eprintln!("\n{}\n", "No sessions found to choose from.".blue().bold());
        std::process::exit(0);
    }

    Ok(FzfCmd::new()
        .args(vec!["--phony", "--multi"])
        .find_vec(session_names)?
        .trim_end()
        .split('\n')
        .map(|s| s.to_string())
        .collect())
}

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
