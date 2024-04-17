use anyhow::Result;
use colored::Colorize;
use std::{
    ffi::OsStr,
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
}

#[derive(Debug)]
pub struct FzfCmd {
    command: Command,
}

impl Default for FzfCmd {
    fn default() -> Self {
        Self::new()
    }
}

impl FzfCmd {
    pub fn new() -> Self {
        Self {
            command: Command::new("fzf"),
        }
    }

    #[instrument(skip(self))]
    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr> + Debug,
    {
        self.command.arg(arg);
        self
    }

    #[instrument(skip(self))]
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S> + Debug,
        S: AsRef<OsStr> + Debug,
    {
        self.command.args(args);
        self
    }

    #[instrument(skip(self))]
    pub fn find_vec<T>(&mut self, input: Vec<T>) -> Result<String>
    where
        T: Debug + Display,
    {
        let projects_string: String = input.iter().fold(String::new(), |acc, project_name| {
            format!("{acc}\n{project_name}")
        });
        self.find_string(projects_string.trim_start())
    }

    #[instrument(skip(self))]
    pub fn find_string(&mut self, input: &str) -> Result<String> {
        let mut fzf_child = self
            .command
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

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }

        Ok("".to_string())
    }

    #[instrument(err)]
    pub fn pick_many(items: Vec<String>) -> Result<Vec<String>> {
        if items.is_empty() {
            eprintln!("\n{}\n", "No items found to choose from.".blue().bold());
            Err(FzfError::NoItemsFound)?
        }

        Ok(Self::new()
            .args(vec!["--phony", "--multi"])
            .find_vec(items)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .collect())
    }

    #[instrument(err)]
    pub fn pick_many_filtered(items: Vec<String>) -> Result<Vec<String>> {
        if items.is_empty() {
            eprintln!("\n{}\n", "No items found to choose from.".blue().bold());
            Err(FzfError::NoItemsFound)?
        }

        Ok(Self::new()
            .arg("--multi")
            .find_vec(items)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .collect())
    }

    #[instrument(err)]
    pub fn pick_one_filtered(items: Vec<String>) -> Result<String> {
        if items.is_empty() {
            eprintln!("\n{}\n", "No items found to choose from.".blue().bold());
            Err(FzfError::NoItemsFound)?
        }

        let picked: Vec<_> = Self::new()
            .find_vec(items)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .collect();

        Ok(picked.first().expect("you must choose one item").clone())
    }
}
