use clap::{Args, Subcommand, ValueEnum};
use git_lib::repo::GitRepo;

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub project_folder_path: PathBuf,
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,
    pub remote: Option<String>,
}

impl Project {
    pub fn new(path: &Path, name: String, remote: Option<String>) -> Self {
        Self {
            project_folder_path: path.to_path_buf(),
            path: path.join(name.clone()),
            name: name.clone(),
            safe_name: name.replace('.', "_"),
            remote,
        }
    }

    pub fn get_safe_name(&self) -> String {
        self.safe_name.clone()
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
    config::AxlContext,
    config_env::ConfigEnvKey,
    helper::fzf_get_sessions,
    multiplexer::{Multiplexer, Multiplexers},
    project_manager::ProjectManager,
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
        sess_args: SessionArgs,
    },
    /// Open new unique session in $HOME and increment prefix (available: 0-9).
    Home {
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// List all projects tracked in your config list.
    List {
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Debug)]
        output: OutputFormat,
    },
    /// Show a report of projects
    ///
    /// This will show you projects tracked in your config file, and the projects in your project
    /// directory that are not tracked.
    Report,
    /// Clone a new repo into your projects dir.
    New {
        ssh_uri: String,
    }, // Like ThePrimagen Harpoon in nvim but for multiplexer sessions
    // Harpoon(ProjectArgs),
    Test,
    /// Reconsile projects defined in config with projects in the directory.
    ///
    /// This will not be descructive. It will only add projects from config that are not already in project folder.
    /// if you want to remove a project you should remove it from your config, and then manually
    /// remove it from the file
    Sync,
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
    pub fn handle_cmd(
        project_sub_cmd: Self,
        projects_dir: PathBuf,
        context: AxlContext,
    ) -> anyhow::Result<()> {
        let project_manager = ProjectManager::new(&projects_dir, context.config.project);
        match project_sub_cmd {
            Self::Open {
                proj_args,
                sess_args,
            } => {
                let project =
                    project_manager.get_project(&proj_args.project_dir, proj_args.name.clone())?;
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
                        None,
                    ),
                )?;
                Ok(())
            }
            Self::Kill { sess_args } => {
                let sessions = sess_args.multiplexer.get_sessions();
                log::debug!("sessions: {sessions:?}");
                let picked_sessions = fzf_get_sessions(sessions)?;
                let current_session = sess_args.multiplexer.get_current_session();
                log::debug!("current session: {current_session}");
                sess_args
                    .multiplexer
                    .kill_sessions(picked_sessions, &current_session)?;
                Ok(())
            }
            Self::Home { sess_args } => sess_args.multiplexer.unique_session(),
            Self::New { ssh_uri } => {
                log::debug!("Attempting to clone {ssh_uri}...");
                let results = GitRepo::from_ssh_uri_multi(&[&ssh_uri], &projects_dir);
                for result in results {
                    if let Err(err) = result {
                        log::error!("Failed cloning with: {err:?}");
                    }
                }
                Ok(())
            }
            Self::Report => {
                let projects_fs = project_manager.get_projects_from_fs()?;
                let projects_config = project_manager.get_projects_from_config()?;
                let filtered = projects_fs
                    .iter()
                    .filter(|p| {
                        !projects_config
                            .iter()
                            .map(|p_c| p_c.name.clone())
                            .any(|x| x == p.name)
                    })
                    .collect::<Vec<_>>();
                println!(
                    "PROJECTS REPORT ({})",
                    project_manager.root_dir.to_string_lossy()
                );
                println!("===============");
                println!(
                    "file system: {}\nconfig list: {}\nnot tracked: {}",
                    projects_fs.len(),
                    projects_config.len(),
                    filtered.len(),
                );
                println!("projects in file system not tracked in config list: ");
                println!("{:#?}", filtered.iter().collect::<Vec<_>>());
                Ok(())
            }
            Self::List { output } => {
                let projects = project_manager.get_projects_from_config()?;
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
            Self::Sync => Ok(()),
            Self::Test => Ok(()),
        }
    }
}
