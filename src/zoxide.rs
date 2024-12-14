use std::{fmt::Debug, path::PathBuf, process::Command};
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
}
