use std::path::Path;

use anyhow::Result;
use clap::ValueEnum;
use strum::Display;
use tracing::instrument;

use self::tmux::TmuxCmd;

pub mod tmux;

pub trait Multiplexer {
    fn open(self, path: &Path, name: &str) -> Result<()>;
    fn open_existing(self, name: &str) -> Result<()>;
    fn open_existing_cmd(self, name: &str) -> String;
    fn get_sessions(self) -> Result<Vec<String>>;
    fn get_current_session(self) -> String;
    fn kill_sessions(self, sessions: Vec<String>, current_session: &str) -> Result<()>;
    fn kill_session_cmd(self, session: &str) -> String;
    fn unique_session(self) -> Result<()>;
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Multiplexers {
    Tmux,
}

impl Multiplexers {
    pub fn as_arg(self) -> String {
        self.to_string().to_lowercase()
    }
}

impl Multiplexer for Multiplexers {
    #[instrument(skip_all, err)]
    fn open(self, path: &Path, name: &str) -> Result<()> {
        match self {
            Self::Tmux => {
                TmuxCmd::open_new(path, name)?;
            }
        }
        Ok(())
    }

    fn open_existing_cmd(self, name: &str) -> String {
        match self {
            Self::Tmux => TmuxCmd::switch_cmd(name).into_string(),
        }
    }

    #[instrument(skip_all, err)]
    fn open_existing(self, name: &str) -> Result<()> {
        match self {
            Self::Tmux => {
                TmuxCmd::open_existing(name)?;
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    fn get_sessions(self) -> Result<Vec<String>> {
        match self {
            Self::Tmux => TmuxCmd::list_sessions(),
        }
    }

    #[instrument(skip_all)]
    fn get_current_session(self) -> String {
        match self {
            Self::Tmux => TmuxCmd::get_current_session(),
        }
    }

    fn kill_session_cmd(self, session: &str) -> String {
        match self {
            Self::Tmux => TmuxCmd::kill_session_cmd(session).into_string(),
        }
    }

    #[instrument(skip(self), err)]
    fn kill_sessions(self, sessions: Vec<String>, current_session: &str) -> Result<()> {
        match self {
            Self::Tmux => TmuxCmd::kill_sessions(&sessions, current_session),
        }
    }

    #[instrument(skip_all, err)]
    fn unique_session(self) -> Result<()> {
        match self {
            Self::Tmux => TmuxCmd::unique_session(),
        }
    }
}
