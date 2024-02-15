use anyhow::Result;
use clap::ValueEnum;

use crate::subcommand_project::{Project, ProjectArgs};

use self::tmux::Tmux;

pub mod tmux;

pub trait Multiplexer {
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()>;
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
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()> {
        match self {
            Self::Tmux => {
                Tmux::open(proj_args, project)?;
            }
        }
        Ok(())
    }

    fn get_sessions(self) -> Vec<String> {
        match self {
            Self::Tmux => Tmux::list_sessions(),
        }
    }

    fn get_current_session(self) -> String {
        match self {
            Self::Tmux => Tmux::get_current_session(),
        }
    }

    fn kill_sessions(self, sessions: Vec<String>, current_session: &str) -> Result<()> {
        match self {
            Self::Tmux => Tmux::kill_sessions(&sessions, current_session),
        }
    }

    fn unique_session(self) -> Result<()> {
        match self {
            Self::Tmux => Tmux::unique_session(),
        }
    }
}
