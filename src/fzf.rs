use crate::multiplexer::{Multiplexer, Multiplexers};
use anyhow::Result;
use colored::Colorize;
use std::{
    ffi::OsStr,
    fmt::{Debug, Display},
    io::Write,
    process::{Command, Stdio},
};
use thiserror::Error;
use tracing::{debug, instrument, trace};

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
struct FzfKeyBind {
    key: String,
    cmd: String,
    silent: bool,
    reload_cmd: Option<String>,
    description: String,
}

impl FzfKeyBind {
    fn new_silent_reloaded(key: &str, cmd: &str, reload: &str, description: &str) -> Self {
        Self {
            key: key.to_string(),
            cmd: cmd.to_string(),
            silent: true,
            reload_cmd: Some(reload.to_string()),
            description: description.to_string(),
        }
    }

    fn build_binds_and_headers(binds: Vec<Self>) -> (String, String) {
        binds.iter().fold(
            (String::new(), String::new()),
            |(mut acc_cmd, mut acc_hdr), bind| {
                acc_cmd = format!(
                    "{}:execute{silent}({cmd}){reload}{acc}",
                    bind.key,
                    silent = if bind.silent { "-silent" } else { "" },
                    cmd = bind.cmd,
                    reload = bind.reload_cmd.clone().map_or_else(
                        || "".to_string(),
                        |reload_cmd| format!("+reload({reload_cmd})")
                    ),
                    acc = if acc_cmd.is_empty() {
                        "".to_string()
                    } else {
                        format!(",{}", acc_cmd)
                    },
                );

                acc_hdr = format!(
                    "{key}: {desc}{acc}",
                    key = bind.key,
                    desc = bind.description,
                    acc = if acc_hdr.is_empty() {
                        "".to_string()
                    } else {
                        format!(", {}", acc_hdr)
                    }
                );

                (acc_cmd, acc_hdr)
            },
        )
    }
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

    pub fn add_custom_keys(&mut self, multiplexer: Multiplexers) -> Result<&mut Self> {
        let bin = std::env::current_exe()
            .expect("the current executable path to be available")
            .to_string_lossy()
            .to_string();
        let (bind_str, header_str) = FzfKeyBind::build_binds_and_headers(vec![
            FzfKeyBind::new_silent_reloaded(
                "ctrl-k",
                &multiplexer.kill_session_cmd("{}"),
                &format!("{} ls --multiplexer={}", bin, multiplexer.as_arg()),
                "kill session under cursor",
            ),
            // TODO: make this multiplexer agnostic
            FzfKeyBind::new_silent_reloaded(
                "ctrl-o",
                &multiplexer.open_existing_cmd("{}"),
                &format!("{} ls --multiplexer={}", bin, multiplexer.as_arg()),
                "(or Enter) open session under cursor",
            ),
        ]);
        trace!("binds: {bind_str}");
        self.command
            .arg("--bind")
            .arg(bind_str) // TODO: reload the sessions from axl
            .arg("--header")
            .arg(format!("[{}]", header_str));

        Ok(self)
    }

    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr> + Debug,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S> + Debug,
        S: AsRef<OsStr> + Debug,
    {
        self.command.args(args);
        self
    }

    #[instrument(skip(self))]
    pub fn find_vec<T>(&mut self, input: Vec<T>) -> Result<String, FzfError>
    where
        T: Debug + Display,
    {
        let projects_string: String = input.iter().fold(String::new(), |acc, project_name| {
            format!("{acc}\n{project_name}")
        });
        self.find_string(projects_string.trim_start())
    }

    #[instrument(skip(self))]
    pub fn find_string(&mut self, input: &str) -> Result<String, FzfError> {
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
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(FzfError::NoItemSelected)
        }
    }

    #[instrument(err)]
    pub fn pick_many(items: Vec<String>) -> Result<Vec<String>> {
        if items.is_empty() {
            eprintln!("\n{}\n", "No items found to choose from.".blue().bold());
            Err(FzfError::NoItemsFound)?
        }

        debug!("pickable_items: {items:?}");

        let picked_items = Self::new()
            .args(vec!["--phony", "--multi"])
            .find_vec(items)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .collect();

        debug!("picked_items: {picked_items:?}");

        Ok(picked_items)
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
    pub fn pick_one_filtered(&mut self, items: Vec<String>) -> Result<String> {
        if items.is_empty() {
            eprintln!("\n{}\n", "No items found to choose from.".blue().bold());
            Err(FzfError::NoItemsFound)?
        }

        let picked: Vec<_> = self
            .find_vec(items)?
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
