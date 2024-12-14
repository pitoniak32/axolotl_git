use anyhow::Result;
use colored::Colorize;
use std::{
    fmt::{Debug, Display},
    io::Write,
    process::{Command, Stdio},
};
use thiserror::Error;
use tracing::instrument;

#[derive(Error, Debug)]
pub enum FzfError {
    #[error("could not find any items to choose from")]
    NoItemsFound,
    #[error("no item selected from options")]
    NoItemSelected,
    #[error("waiting on fzf command failed")]
    CommandFailed(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct FzfCmd;

impl FzfCmd {
    const CMD: &'static str = "fzf";

    #[instrument()]
    pub fn find_vec<T>(input: Vec<T>) -> Result<String, FzfError>
    where
        T: Debug + Display,
    {
        let projects_string: String = input.iter().fold(String::new(), |acc, project_name| {
            format!("{acc}\n{project_name}")
        });
        Self::find_string(projects_string.trim_start())
    }

    /// Will always return the query-string if no item is selected.
    #[instrument()]
    pub fn find_string(input: &str) -> Result<String, FzfError> {
        let mut fzf_child = Command::new(Self::CMD)
            .arg("--tmux")
            .arg("--print-query")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("fzf command should spawn");

        // Get the stdin handle of the child process
        fzf_child.stdin.as_mut().map_or_else(
            || {
                eprintln!("Failed to get stdin handle for the child process");
            },
            |stdin| {
                // Write your input string to the command's stdin
                stdin
                    .write_all(input.as_bytes())
                    .expect("should be able to pass project names to fzf stdin");
            },
        );

        // Ensure the child process has finished
        let output = fzf_child.wait_with_output()?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if output.status.success() && result.contains("\n") {
            let parts = result.split("\n").collect::<Vec<_>>();
            tracing::debug!("fzf success: {:?}", parts);
            return Ok(parts.get(1).expect("when the command is successful the result should contain query-string, and choosen option").to_string());
        };

        // Still return the query string
        Ok(result)
    }

    #[instrument(err)]
    pub fn pick_one_filtered(&mut self, items: Vec<String>) -> Result<String> {
        if items.is_empty() {
            eprintln!("\n{}\n", "No items found to choose from.".blue().bold());
            Err(FzfError::NoItemsFound)?
        }

        let picked: Vec<_> = Self::find_vec(items)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .filter(|n| !n.is_empty())
            .collect::<Vec<String>>();

        match picked.first() {
            Some(val) => Ok(val.clone()),
            None => Err(FzfError::NoItemSelected)?,
        }
    }
}
