use clap::{Args, Subcommand, ValueEnum};
use git_lib::repo::GitRepo;

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Project {
    pub project_folder_path: PathBuf,
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,
    pub remote: String,
}

impl Project {
    pub fn new(path: &Path, name: String, remote: String) -> Self {
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
    config::{config_env::ConfigEnvKey, config_file::AxlContext},
    helper::fzf_get_sessions,
    multiplexer::{Multiplexer, Multiplexers},
    project::project_directory_manager::ProjectsDirectoryFile,
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

    /// Manually set the project root dir.
    #[arg(long, env)]
    projects_directory_file: PathBuf,

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
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Debug)]
        output: OutputFormat,
    },
    /// Select projects to bring into axl tracking
    ///
    /// This will pick projects, from a specified directory, and give a yaml string to add into your config file.
    Import {
        /// The projects directory to pick from
        #[arg(short, long)]
        directory: PathBuf,
    },
    /// Show a report of projects
    ///
    /// This will show you projects tracked in your config file, and the projects in your project
    /// directory that are not tracked.
    Report {
        #[clap(flatten)]
        proj_args: ProjectArgs,
    },
    /// Clone a new repo into your projects dir.
    New {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        ssh_uri: String,
    }, // Like ThePrimagen Harpoon in nvim but for multiplexer sessions
       // Harpoon(ProjectArgs),
       // Test,
       // Reconsile projects defined in config with projects in the directory.
       //
       // This will not be descructive. It will only add projects from config that are not already in project folder.
       // if you want to remove a project you should remove it from your config, and then manually
       // /// remove it from the file
       // Sync,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    /// rust debug print.
    Debug,
    /// pretty printed json.
    Json,
    /// raw printed json.
    JsonR,
    /// yaml.
    Yaml,
    /// csv for excel spreadsheets.
    Csv,
}

impl ProjectSubcommand {
    pub fn handle_cmd(project_sub_cmd: Self, _context: AxlContext) -> anyhow::Result<()> {
        match project_sub_cmd {
            Self::Open {
                proj_args,
                sess_args,
            } => {
                log::debug!(
                    "using [{:?}] projects file.",
                    proj_args.projects_directory_file
                );
                let projects_directory_file =
                    ProjectsDirectoryFile::new(&proj_args.projects_directory_file)?;
                let project = projects_directory_file.get_project()?;
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
                        "".to_owned(),
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
            Self::New { proj_args, ssh_uri } => {
                log::debug!(
                    "using [{:?}] projects file.",
                    proj_args.projects_directory_file
                );
                let projects_directory_file =
                    ProjectsDirectoryFile::new(&proj_args.projects_directory_file)?;
                log::debug!("Attempting to clone {ssh_uri}...");
                let results =
                    GitRepo::from_ssh_uri_multi(&[&ssh_uri], &projects_directory_file.path);
                for result in results {
                    if let Err(err) = result {
                        log::error!("Failed cloning with: {err:?}");
                    }
                }
                Ok(())
            }
            Self::Report { proj_args } => {
                let projects_directory_file =
                    ProjectsDirectoryFile::new(&proj_args.projects_directory_file)?;
                log::trace!(
                    "getting projects from fs [{}]",
                    &projects_directory_file.path.to_string_lossy()
                );
                let projects_fs =
                    ProjectsDirectoryFile::get_projects_from_fs(&projects_directory_file.path)?;
                log::trace!("got projects from fs [{:#?}]", &projects_fs);
                log::trace!(
                    "getting projects from project_directory_file [{}] remotes",
                    &proj_args.projects_directory_file.to_string_lossy()
                );
                let projects_remotes = projects_directory_file.get_projects_from_remotes()?;
                log::trace!(
                    "got projects from project_directory_file remotes [{:#?}]",
                    &projects_fs
                );
                let filtered = projects_fs
                    .0
                    .iter()
                    .filter(|p| {
                        !projects_remotes
                            .iter()
                            .map(|p_c| p_c.name.clone())
                            .any(|x| x == p.name)
                    })
                    .collect::<Vec<_>>();
                println!(
                    "PROJECTS REPORT ({})",
                    projects_directory_file.path.to_string_lossy()
                );
                println!("===============");
                println!(
                    "file system: {}\nconfig list: {}\nnot tracked: {}\nignored: {}\n",
                    projects_fs.0.len(),
                    projects_remotes.len(),
                    filtered.len(),
                    projects_fs.1.len(),
                );

                if !filtered.is_empty() {
                    println!(
                        "items in [{}] not tracked in config list: ",
                        projects_directory_file.path.to_string_lossy()
                    );
                    println!("{:#?}", filtered.iter().collect::<Vec<_>>());
                }

                if !projects_fs.1.is_empty() {
                    println!(
                        "ignored items in [{}]: ",
                        projects_directory_file.path.to_string_lossy()
                    );
                    println!("{:#?}", projects_fs.1.iter().collect::<Vec<_>>());
                }
                Ok(())
            }
            Self::Import { directory } => {
                let projects = ProjectsDirectoryFile::pick_projects(
                    ProjectsDirectoryFile::get_projects_from_fs(&directory)?.0,
                )?;

                let projects = projects.into_iter().map(|p| p.remote).collect::<Vec<_>>();
                println!(
                    "Copy into your project file list:\n---\n{}",
                    serde_yaml::to_string(&projects)?
                );

                Ok(())
            }
            Self::List { proj_args, output } => {
                let projects_directory_file =
                    ProjectsDirectoryFile::new(&proj_args.projects_directory_file)?;
                let projects = projects_directory_file.get_projects_from_remotes()?;
                match output {
                    OutputFormat::Debug => {
                        println!("{:#?}", projects);
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&projects)?)
                    }
                    OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&projects)?),
                    OutputFormat::Csv => println!(
                        "{},",
                        projects
                            .iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join(",\n")
                    ),
                    OutputFormat::JsonR => {
                        println!("{}", serde_json::to_string(&projects)?)
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::Result;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    use crate::subcommand_project::Project;

    #[rstest]
    fn should_create_new_project() -> Result<()> {
        // Act
        let project = Project::new(
            &PathBuf::from("/test/projects/dir/"),
            "test2".to_string(),
            "git@github.com:user/test2.git".to_string(),
        );

        // Assert
        assert_eq!(
            project,
            Project {
                name: "test2".to_string(),
                safe_name: "test2".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string()
            }
        );

        Ok(())
    }
}
