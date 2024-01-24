use clap::{Args, Subcommand, ValueEnum};
use git_lib::repo::GitRepo;

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
}

impl Project {
    pub fn new(path: &Path, name: String) -> Self {
        Self {
            path: path.to_path_buf(),
            name: name.replace('.', "_"),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

use crate::{
    config_env::ConfigEnvKey,
    helper::{fzf_get_sessions, get_project, get_projects},
    multiplexer::{Multiplexer, Multiplexers},
};

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    /// Which multiplexer session should be created.
    pub multiplexer: Multiplexers,
}

#[derive(Args, Debug)]
pub struct ProjectArgs {
    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    pub name: Option<String>,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    pub project_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ProjectSubcommand {
    /// Open a session.
    Open {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Open a scratch session. defaults: (name = scratch, path = $HOME)
    Scratch {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Kill sessions.
    Kill {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Open new unique session in $HOME and increment prefix (available: 0-9).
    Home {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// List all projects in your projects dir.
    List {
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Debug)]
        output: OutputFormat,
    },
    /// Clone a new repo into your projects dir.
    New {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        ssh_uri: String,
    }, // Like ThePrimagen Harpoon in nvim but for multiplexer sessions
       // Harpoon(ProjectArgs),
}

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    /// Rust Debug print.
    Debug,
    /// Pretty printed json.
    Json,
    /// Raw printed json.
    JsonR,
    /// yaml.
    Yaml,
}

impl ProjectSubcommand {
    pub fn handle_cmd(project_sub_cmd: Self, projects_dir: PathBuf) -> anyhow::Result<()> {
        match project_sub_cmd {
            Self::Open {
                proj_args,
                sess_args,
            } => {
                let project =
                    get_project(projects_dir, &proj_args.project_dir, proj_args.name.clone())?;
                sess_args.multiplexer.open(&proj_args, project)?;
                Ok(())
            }
            Self::Scratch {
                proj_args,
                sess_args,
            } => {
                sess_args.multiplexer.open(
                    &proj_args,
                    Project::new(
                        &proj_args
                            .project_dir
                            .clone()
                            .unwrap_or(PathBuf::try_from(ConfigEnvKey::Home)?),
                        proj_args
                            .name
                            .clone()
                            .unwrap_or_else(|| "scratch".to_string()),
                    ),
                )?;
                Ok(())
            }
            Self::Kill {
                proj_args: _,
                sess_args,
            } => {
                let sessions = sess_args.multiplexer.get_sessions();
                log::debug!("sessions: {sessions:?}");
                let picked_sessions = fzf_get_sessions(sessions)?;
                sess_args.multiplexer.kill_sessions(picked_sessions)?;
                Ok(())
            }
            Self::Home {
                proj_args: _,
                sess_args,
            } => sess_args.multiplexer.unique_session(),
            Self::New {
                proj_args: _,
                ssh_uri,
            } => {
                log::debug!("Attempting to clone {ssh_uri}...");
                let results = GitRepo::from_ssh_uri_multi(&[&ssh_uri], &projects_dir);
                for result in results {
                    if let Err(err) = result {
                        log::error!("Failed cloning with: {err:?}");
                    }
                }
                Ok(())
            }
            Self::List { output } => {
                let projects = get_projects(&projects_dir)?;
                match output {
                    OutputFormat::Debug => {
                        println!("{:#?}", projects);
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&projects)?)
                    }
                    OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&projects)?),
                    OutputFormat::JsonR => {
                        println!("{}", serde_json::to_string(&projects)?)
                    }
                }
                Ok(())
            }
        }
    }
}
