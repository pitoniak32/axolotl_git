use std::{collections::BTreeSet, fs, path::PathBuf};

use clap::{arg, Args, Subcommand, ValueEnum};
use colored::Colorize;
use git_lib::{git::Git, repo::GitRepo};
use tracing::{debug, error, instrument, trace};

use crate::{
    config::{config_env::ConfigEnvKey, config_file::AxlContext},
    helper::fzf_get_sessions,
    multiplexer::{Multiplexer, Multiplexers},
    project::{
        project_directory::{ConfigProjectDirectory, ResolvedProjectDirectory},
        project_type::{ConfigProject, ResolvedProject},
    },
};

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    /// Which multiplexer session should be created.
    pub multiplexer: Multiplexers,
}

#[derive(Args, Debug)]
pub struct ProjectArgs {
    /// Manually set the project root dir.
    #[arg(long, env)]
    projects_directory_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct FilterArgs {
    /// Comma delimited list of tags narrowing projects that will be operated on.
    #[arg(long, short, value_delimiter = ',')]
    tags: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ProjectSubcommand {
    /// Open a session.
    Open {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        filter_args: FilterArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Open a scratch session. defaults: (name = scratch, path = $HOME)
    Scratch {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
        #[arg(short, long)]
        /// Name of session, defaults to project_dir name
        name: Option<String>,
        #[arg(short, long)]
        /// Name of session, defaults to project_dir name
        project_dir: Option<PathBuf>,
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
        #[clap(flatten)]
        filter_args: FilterArgs,
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Debug)]
        output: OutputFormat,
    },
    /// List all tags used on projects tracked in your config list.
    ListTags {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Debug)]
        output: OutputFormat,
    },
    /// Select projects to bring into axl tracking
    ///
    /// This will pick projects, from a specified directory, and give a yaml string to add into your config file.
    Import {
        #[clap(flatten)]
        proj_args: ProjectArgs,
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
        #[clap(flatten)]
        filter_args: FilterArgs,
    },
    /// Clone a new repo into your projects dir.
    New {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        /// If the repo should be initialized in the project directory
        #[arg(long)]
        init: bool,
        /// remote uri of repository you would like to add
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
    #[instrument(skip(project_sub_cmd, _context), err)]
    pub fn handle_cmd(project_sub_cmd: Self, _context: AxlContext) -> anyhow::Result<()> {
        match project_sub_cmd {
            Self::Open {
                proj_args,
                filter_args,
                sess_args,
            } => {
                debug!(
                    "using [{:?}] projects file.",
                    proj_args.projects_directory_file
                );
                let projects_directory_file = ResolvedProjectDirectory::new_filtered(
                    &ConfigProjectDirectory::new(&proj_args.projects_directory_file)?,
                    &filter_args.tags,
                )?;
                let project = projects_directory_file.get_project()?;
                sess_args.multiplexer.open(&proj_args, project)?;
                Ok(())
            }
            Self::Scratch {
                proj_args,
                sess_args,
                name,
                project_dir,
            } => {
                sess_args.multiplexer.open(
                    &proj_args,
                    ResolvedProject::new(
                        &project_dir.unwrap_or(PathBuf::try_from(ConfigEnvKey::Home)?),
                        name.unwrap_or_else(|| "scratch".to_string()),
                        "".to_owned(),
                        BTreeSet::new(),
                    ),
                )?;
                Ok(())
            }
            Self::Kill { sess_args } => {
                let sessions = sess_args.multiplexer.get_sessions();
                debug!("sessions: {sessions:?}");
                let picked_sessions = fzf_get_sessions(sessions)?;
                let current_session = sess_args.multiplexer.get_current_session();
                debug!("current session: {current_session}");
                sess_args
                    .multiplexer
                    .kill_sessions(picked_sessions, &current_session)?;
                Ok(())
            }
            Self::Home { sess_args } => sess_args.multiplexer.unique_session(),
            Self::New {
                proj_args,
                init,
                ssh_uri,
            } => {
                debug!(
                    "using [{:?}] projects file.",
                    proj_args.projects_directory_file
                );
                let mut project_directory = ResolvedProjectDirectory::new(
                    &ConfigProjectDirectory::new(&proj_args.projects_directory_file)?,
                )?;
                if project_directory
                    .projects
                    .iter()
                    .filter(|config_proj| config_proj.remote == ssh_uri)
                    .count()
                    > 0
                {
                    eprintln!(
                        "{}",
                        "Project with this remote already exists in your projects_directory_file"
                            .red()
                            .bold()
                    );
                    return Ok(());
                }
                debug!("Attempting to clone {ssh_uri}...");
                if init {
                    let parsed = Git::parse_url(&ssh_uri)?;
                    let project_dir = project_directory.projects_directory.join(parsed.name);

                    if !project_dir.exists() {
                        fs::create_dir(&project_dir)?;
                    }

                    if !Git::is_inside_worktree(&project_dir) {
                        Git::init(&project_dir)?;
                        Git::add_remote("origin", &ssh_uri, &project_dir)?;
                    }
                    project_directory.add_config_projects(vec![ConfigProject {
                        name: None,
                        remote: ssh_uri,
                        tags: BTreeSet::new(),
                    }])?;
                } else {
                    let results =
                        GitRepo::from_url_multi(&[&ssh_uri], &project_directory.projects_directory);
                    for result in results {
                        if let Err(err) = result {
                            error!("Failed cloning with: {err:?}");
                        }
                    }
                    project_directory.add_config_projects(vec![ConfigProject {
                        name: None,
                        remote: ssh_uri,
                        tags: BTreeSet::new(),
                    }])?;
                }

                Ok(())
            }
            Self::Report {
                proj_args,
                filter_args,
            } => {
                let filtered_project_directory = ResolvedProjectDirectory::new_filtered(
                    &ConfigProjectDirectory::new(&proj_args.projects_directory_file)?,
                    &filter_args.tags,
                )?;
                trace!(
                    "getting projects from fs [{}]",
                    &filtered_project_directory
                        .projects_directory
                        .to_string_lossy()
                );
                let projects_fs = ResolvedProjectDirectory::get_projects_from_fs(
                    &filtered_project_directory.projects_directory,
                )?;
                trace!("got projects from fs [{:#?}]", &projects_fs);
                trace!(
                    "getting projects from project_directory_file [{}] remotes",
                    &proj_args.projects_directory_file.to_string_lossy()
                );
                let projects_remotes = filtered_project_directory.get_projects_from_remotes()?;
                trace!(
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
                    filtered_project_directory
                        .projects_directory
                        .to_string_lossy()
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
                        filtered_project_directory
                            .projects_directory
                            .to_string_lossy()
                    );
                    println!("{:#?}", filtered.iter().collect::<Vec<_>>());
                }

                if !projects_fs.1.is_empty() {
                    println!(
                        "ignored items in [{}]: ",
                        filtered_project_directory
                            .projects_directory
                            .to_string_lossy()
                    );
                    println!("{:#?}", projects_fs.1.iter().collect::<Vec<_>>());
                }
                Ok(())
            }
            Self::Import {
                proj_args,
                directory,
            } => {
                let mut project_directory_file = ResolvedProjectDirectory::new(
                    &ConfigProjectDirectory::new(&proj_args.projects_directory_file)?,
                )?;

                let existing_projects = project_directory_file
                    .projects
                    .clone()
                    .into_iter()
                    .map(|ep| ep.remote)
                    .collect::<Vec<_>>();

                trace!("existing: {existing_projects:?}");

                let selected_projects = ResolvedProjectDirectory::pick_config_projects(
                    ResolvedProjectDirectory::get_projects_from_fs(&directory)?
                        .0
                        .into_iter()
                        .filter(|sp| !existing_projects.contains(&sp.remote))
                        .map(|sp| ConfigProject {
                            name: None,
                            remote: sp.remote,
                            tags: sp.tags,
                        })
                        .collect::<Vec<_>>(),
                )?;

                trace!("selected: {selected_projects:?}");

                project_directory_file.add_config_projects(selected_projects)?;

                Ok(())
            }
            Self::List {
                proj_args,
                filter_args,
                output,
            } => {
                let filtered_project_directory = ResolvedProjectDirectory::new_filtered(
                    &ConfigProjectDirectory::new(&proj_args.projects_directory_file)?,
                    &filter_args.tags,
                )?;
                let projects = filtered_project_directory.get_projects_from_remotes()?;
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
            Self::ListTags { proj_args, output } => {
                let project_directory = ResolvedProjectDirectory::new(
                    &ConfigProjectDirectory::new(&proj_args.projects_directory_file)?,
                )?;
                let tags = project_directory.get_projects_from_remotes()?.iter().fold(
                    BTreeSet::new(),
                    |mut acc, project| {
                        acc.extend(project.tags.clone());
                        acc
                    },
                );
                match output {
                    OutputFormat::Debug => {
                        println!("{:#?}", tags);
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&tags)?)
                    }
                    OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&tags)?),
                    OutputFormat::Csv => {
                        println!("{},", tags.iter().cloned().collect::<Vec<_>>().join(",\n"))
                    }
                    OutputFormat::JsonR => {
                        println!("{}", serde_json::to_string(&tags)?)
                    }
                }
                Ok(())
            }
        }
    }
}
