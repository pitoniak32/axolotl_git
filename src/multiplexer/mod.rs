use anyhow::Result;
use clap::ValueEnum;
use tracing::instrument;

use crate::project::{project_type::ResolvedProject, subcommand::ProjectArgs};

use self::tmux::Tmux;

pub mod tmux;

pub trait Multiplexer {
    fn open(self, proj_args: &ProjectArgs, project: ResolvedProject) -> Result<()>;
    fn get_sessions(self) -> Vec<String>;
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
    fn open(self, proj_args: &ProjectArgs, project: ResolvedProject) -> Result<()> {
        match self {
            Self::Tmux => {
                Tmux::open(proj_args, project)?;
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    fn get_sessions(self) -> Vec<String> {
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
