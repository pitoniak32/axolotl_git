use anyhow::Result;
use colored::Colorize;
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{config::config_env::ConfigEnvKey, error::Error, helper::wrap_command};

pub struct TmuxCmd {
    pub cmd: String,
    pub args: Vec<String>,
}

impl TmuxCmd {
    const CMD: &'static str = "tmux";

    #[instrument(err)]
    pub fn open(path: &Path, name: &str) -> Result<()> {
        info!(
            "Attempting to open Tmux session with path: {:?}, name: {:?}!",
            path, name,
        );

        if !path.exists() {
            Err(Error::PathDoesNotExist(path.to_string_lossy().to_string()))?;
        }

        if !Self::in_session() {
            Self::create_new_attached_attach_if_exists(name, path)?;
        } else if Self::has_session(name) {
            info!("Session '{name}' already exists, opening.");
            Self::switch(name)?;
        } else {
            info!("Session '{name}' does not already exist, creating and opening.",);

            if Self::create_new_detached(name, path).is_ok_and(|o| o.status.success()) {
                Self::switch(name)?;
            } else {
                eprintln!("{}", "Session failed to open.".red().bold());
                Err(Error::CouldNotCreateSession)?;
            }
        };

        Ok(())
    }

    #[instrument(err)]
    pub fn open_existing(name: &str) -> Result<()> {
        info!(
            "Attempting to open existing Tmux session with name: {:?}!",
            name,
        );

        if !Self::in_session() {
            trace!("Not currently in session, attempting to attach to tmux session",);
            Self::attach()?;
        }

        Self::switch(name)?;

        Ok(())
    }

    #[instrument]
    pub fn list_sessions() -> Result<Vec<String>> {
        Ok(
            String::from_utf8_lossy(&wrap_command(Command::new(Self::CMD).arg("ls"))?.stdout)
                .trim_end()
                .split('\n')
                .map(|s| s.split(':').collect::<Vec<_>>()[0].to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    }

    #[instrument]
    pub fn get_current_session() -> String {
        String::from_utf8_lossy(
            &wrap_command(
                Command::new(Self::CMD)
                    .arg("display-message")
                    .arg("-p")
                    .arg("#S"),
            )
            .expect("tmux should be able to show current session")
            .stdout,
        )
        .trim_end()
        .to_string()
    }

    #[instrument(err)]
    pub fn kill_sessions(sessions: &[String], current_session: &str) -> Result<()> {
        sessions
            .iter()
            .filter(|s| *s != current_session)
            .for_each(|s| {
                if Self::kill_session(s).is_ok() {
                    if s.is_empty() {
                        warn!("No session picked");
                    } else {
                        info!("Killed {}.", s);
                    }
                } else {
                    error!("Error while killing {}.", s)
                }
            });

        if sessions.contains(&current_session.to_string()) {
            debug!("current session [{current_session}] was included to be killed.");

            if Self::kill_session(current_session).is_ok() {
                if current_session.is_empty() {
                    warn!("No session picked");
                } else {
                    info!("Killed {current_session}.");
                }
            } else {
                error!("Error while killing {current_session}.")
            }
        }

        Ok(())
    }

    #[instrument(err)]
    pub fn unique_session() -> Result<()> {
        for i in 0..10 {
            let name = &i.to_string();
            if !Self::has_session(name) {
                if Self::create_new_detached(name, &PathBuf::try_from(ConfigEnvKey::Home)?)
                    .is_ok_and(|o| o.status.success())
                {
                    Self::switch(name)?;
                    break;
                } else {
                    eprintln!("{}", "Session failed to open.".red().bold());
                }
            }
        }
        Ok(())
    }
}

impl TmuxCmd {
    #[allow(dead_code)] // This will likely be needed eventually.
    #[instrument(err)]
    fn create_new_detached_attach_if_exists(name: &str, path: &Path) -> Result<Output> {
        wrap_command(
            Command::new(Self::CMD)
                .arg("new-session")
                .arg("-Ad")
                .arg("-s")
                .arg(name)
                .arg("-c")
                .arg(path.to_str().unwrap_or_default()),
        )
    }

    #[instrument(err)]
    fn create_new_attached_attach_if_exists(name: &str, path: &Path) -> Result<Output> {
        wrap_command(
            Command::new(Self::CMD)
                .arg("new-session")
                .arg("-A")
                .arg("-s")
                .arg(name)
                .arg("-c")
                .arg(path.to_str().unwrap_or_default()),
        )
    }

    #[instrument(err)]
    fn create_new_detached(name: &str, path: &Path) -> Result<Output> {
        wrap_command(
            Command::new(Self::CMD)
                .arg("new-session")
                .arg("-d")
                .arg("-s")
                .arg(name)
                .arg("-c")
                .arg(path.to_str().unwrap_or_default()),
        )
    }

    #[instrument(err)]
    fn switch(to_name: &str) -> Result<Output> {
        wrap_command(
            Command::new(Self::CMD)
                .arg("switch-client")
                .arg("-t")
                .arg(to_name),
        )
    }

    #[instrument(err)]
    fn attach() -> Result<Output> {
        wrap_command(Command::new(Self::CMD).arg("attach"))
    }

    #[instrument]
    fn has_session(project_name: &str) -> bool {
        wrap_command(
            Command::new(Self::CMD)
                .arg("has-session")
                .arg("-t")
                .arg(project_name),
        )
        .is_ok_and(|o| o.status.success())
    }

    #[instrument(err)]
    fn kill_session(project_name: &str) -> Result<()> {
        wrap_command(
            Command::new(Self::CMD)
                .arg("kill-session")
                .arg("-t")
                .arg(project_name),
        )?;
        Ok(())
    }

    #[instrument]
    fn in_session() -> bool {
        env::var("TMUX").is_ok()
    }
}
