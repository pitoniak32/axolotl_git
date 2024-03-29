use anyhow::Result;
use std::{
    ffi::OsStr,
    fmt::{Debug, Display},
    io::Write,
    process::{Command, Stdio},
};
use tracing::instrument;

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
    #[instrument]
    pub fn new() -> Self {
        Self {
            command: Command::new("fzf"),
        }
    }

    #[instrument(skip(self))]
    pub fn _arg<S>(&mut self, arg: S) -> &mut Self
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
}
