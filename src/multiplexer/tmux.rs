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
    fn run(self) -> Result<Output> {
        wrap_command(Command::new(Self::CMD).arg(self.cmd).args(self.args))
    }

    pub fn into_string(&self) -> String {
        format!(
            "{:?}",
            Command::new(Self::CMD)
                .arg(self.cmd.clone())
                .args(self.args.clone())
        )
        .replace('"', "")
    }
}

impl TmuxCmd {
    const CMD: &'static str = "tmux";

    pub fn open_cmd(path: &Path, name: &str) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::ProjectPathDoesNotExist(
                path.to_string_lossy().to_string(),
            ));
        }

        if !Self::in_session() {
            Ok(Self::create_new_attached_attach_if_exists_cmd(name, path))
        } else if Self::has_session(name) {
            info!("Session '{name}' already exists, opening.");
            Ok(Self::switch_cmd(name))
        } else {
            info!("Session '{name}' does not already exist, creating and opening.",);

            if Self::create_new_detached(name, path).is_ok_and(|o| o.status.success()) {
                Ok(Self::switch_cmd(name))
            } else {
                eprintln!("{}", "Session failed to open.".red().bold());
                Err(Error::CouldNotCreateSession)
            }
        }
    }

    #[instrument(err)]
    pub fn open(path: &Path, name: &str) -> Result<()> {
        info!(
            "Attempting to open Tmux session with path: {:?}, name: {:?}!",
            path, name,
        );

        Self::open_cmd(path, name)?.run()?;

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

        Self::switch_cmd(name).run()?;

        Ok(())
    }

    #[instrument]
    pub fn list_sessions() -> Result<Vec<String>> {
        Ok(
            String::from_utf8_lossy(&wrap_command(Command::new("tmux").arg("ls"))?.stdout)
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
                Command::new("tmux")
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
    pub fn create_new_detached_attach_if_exists_cmd(name: &str, path: &Path) -> Self {
        Self {
            cmd: "new-session".to_string(),
            args: vec![
                "-Ad".to_string(),
                "-s".to_string(),
                name.to_string(),
                "-c".to_string(),
                path.to_str().unwrap_or_default().to_string(),
            ],
        }
    }

    #[allow(dead_code)] // This will likely be needed eventually.
    #[instrument(err)]
    fn create_new_detached_attach_if_exists(name: &str, path: &Path) -> Result<Output> {
        Self::create_new_detached_attach_if_exists_cmd(name, path).run()
    }

    pub fn create_new_attached_attach_if_exists_cmd(name: &str, path: &Path) -> Self {
        Self {
            cmd: "new-session".to_string(),
            args: vec![
                "-A".to_string(),
                "-s".to_string(),
                name.to_string(),
                "-c".to_string(),
                path.to_str().unwrap_or_default().to_string(),
            ],
        }
    }

    #[instrument(err)]
    fn create_new_attached_attach_if_exists(name: &str, path: &Path) -> Result<Output> {
        Self::create_new_attached_attach_if_exists_cmd(name, path).run()
    }

    pub fn create_new_detached_cmd(name: &str, path: &Path) -> Self {
        Self {
            cmd: "new-session".to_string(),
            args: vec![
                "-d".to_string(),
                "-s".to_string(),
                name.to_string(),
                "-c".to_string(),
                path.to_str().unwrap_or_default().to_string(),
            ],
        }
    }

    #[instrument(err)]
    fn create_new_detached(name: &str, path: &Path) -> Result<Output> {
        Self::create_new_detached_cmd(name, path).run()
    }

    pub fn switch_cmd(to_name: &str) -> Self {
        Self {
            cmd: "switch-client".to_string(),
            args: vec!["-t".to_string(), to_name.to_string()],
        }
    }

    #[instrument(err)]
    fn switch(to_name: &str) -> Result<Output> {
        wrap_command(Command::new("tmux").args(["switch-client", "-t", to_name]))
    }

    pub fn attach_cmd() -> Self {
        Self {
            cmd: "attach".to_string(),
            args: vec![],
        }
    }

    #[instrument(err)]
    fn attach() -> Result<Output> {
        Self::attach_cmd().run()
    }

    #[instrument]
    pub fn has_session_cmd(project_name: &str) -> Self {
        Self {
            cmd: "has-session".to_string(),
            args: vec!["-t".to_string(), format!("={}", project_name)],
        }
    }

    #[instrument]
    fn has_session(project_name: &str) -> bool {
        let output = Self::has_session_cmd(project_name).run();

        output.is_ok_and(|o| o.status.success())
    }

    pub fn kill_session_cmd(session_name: &str) -> Self {
        Self {
            cmd: "kill-session".to_string(),
            args: vec!["-t".to_string(), format!("{}", session_name)],
        }
    }

    #[instrument(err)]
    fn kill_session(project_name: &str) -> Result<()> {
        Self::kill_session_cmd(project_name).run()?;
        Ok(())
    }

    #[instrument]
    fn in_session() -> bool {
        env::var("TMUX").is_ok()
    }
}
