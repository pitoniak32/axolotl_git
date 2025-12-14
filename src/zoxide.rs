use std::{
    fmt::Debug,
    path::PathBuf,
    process::{Command, Stdio},
};
use tracing::instrument;

use crate::helper::wrap_command;

#[derive(Debug)]
pub struct ZoxideCmd;

impl ZoxideCmd {
    const CMD: &'static str = "zoxide";

    /// Will always return the query-string if no item is selected.
    #[instrument()]
    pub fn query(input: &str) -> anyhow::Result<PathBuf> {
        let output = wrap_command(Command::new(Self::CMD).arg("query").arg(input))?;

        Ok(PathBuf::from(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    }

    #[instrument()]
    pub fn query_interactive(input: &str) -> anyhow::Result<PathBuf> {
        let fzf_opts = [
            "--tmux",                    // start fzf in a tmux popup
            &format!("--query={input}"), // start fzf with the query pre filled
        ];

        let zoxide_child = Command::new(Self::CMD)
            .env("_ZO_FZF_OPTS", fzf_opts.join(" "))
            .arg("query")
            .arg("--interactive")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("zoxide command should spawn");

        // Ensure the child process has finished
        let output = zoxide_child.wait_with_output()?;

        // Still return the query string
        Ok(PathBuf::from(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    }
}
