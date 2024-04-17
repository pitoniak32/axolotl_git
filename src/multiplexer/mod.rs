use std::path::Path;

use anyhow::Result;
use clap::ValueEnum;
use tracing::instrument;

use self::tmux::Tmux;

pub mod tmux;

pub trait Multiplexer {
    fn open(self, path: &Path, name: &str) -> Result<()>;
    fn open_existing(self, name: &str) -> Result<()>;
    fn get_sessions(self) -> Result<Vec<String>>;
    fn get_current_session(self) -> String;
    fn kill_sessions(self, sessions: Vec<String>, current_session: &str) -> Result<()>;
    fn unique_session(self) -> Result<()>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Multiplexers {
    Tmux,
}

impl Multiplexer for Multiplexers {
    #[instrument(skip_all, err)]
    fn open(self, path: &Path, name: &str) -> Result<()> {
        match self {
            Self::Tmux => {
                Tmux::open(path, name)?;
            }
        }
        Ok(())
    }

    #[instrument(skip_all, err)]
    fn open_existing(self, name: &str) -> Result<()> {
        match self {
            Self::Tmux => {
                Tmux::open_existing(name)?;
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    fn get_sessions(self) -> Result<Vec<String>> {
        match self {
            Self::Tmux => Tmux::list_sessions(),
        }
    }

    #[instrument(skip_all)]
    fn get_current_session(self) -> String {
        match self {
            Self::Tmux => Tmux::get_current_session(),
        }
    }

    #[instrument(skip(self), err)]
    fn kill_sessions(self, sessions: Vec<String>, current_session: &str) -> Result<()> {
        match self {
            Self::Tmux => Tmux::kill_sessions(&sessions, current_session),
        }
    }

    #[instrument(skip_all, err)]
    fn unique_session(self) -> Result<()> {
        match self {
            Self::Tmux => Tmux::unique_session(),
        }
    }
}
