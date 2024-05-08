use std::{collections::BTreeSet, path::PathBuf};

use clap::{arg, Args, Subcommand, ValueEnum};
use colored::Colorize;
use git_lib::repo::GitRepo;
use inquire::{validator::Validation, Confirm, Text};
use tracing::{debug, error, instrument, trace};

use crate::{
    config::{
        config_env::ConfigEnvKey,
        config_file::AxlContext,
        constants::{DEFAULT_MULTIPLEXER_KEY, DEFAULT_PROJECTS_CONFIG_PATH_KEY},
    },
    fzf::FzfCmd,
    helper::formatted_print,
    multiplexer::{Multiplexer, Multiplexers},
    project::{
        project_file::{ConfigProjectDirectory, ResolvedProjectDirectory},
        project_type::ConfigProject,
    },
};

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long, env(DEFAULT_MULTIPLEXER_KEY))]
    /// Which multiplexer should be used for session creation.
    pub multiplexer: Multiplexers,
}

#[derive(Args, Debug)]
pub struct ProjectArgs {
    /// Manually set the project root dir.
    #[arg(long, env(DEFAULT_PROJECTS_CONFIG_PATH_KEY))]
    projects_config_path: PathBuf,
}

#[derive(Args, Debug)]
pub struct FilterArgs {
    /// Comma delimited list of tags narrowing projects that will be operated on.
    #[arg(long, short, value_delimiter = ',')]
    tags: Vec<String>,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub struct ScratchDirArgs {
    /// If not provided you will be prompted for a directory
    #[arg(short, long)]
    project_dir: Option<PathBuf>,
    /// Ignore project directory, and use $HOME
    #[arg(long)]
    home: bool,
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
        #[arg(short, long)]
        existing: bool,
    },
    /// Open a scratch session. defaults: (name = scratch, path = $HOME)
    Scratch {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
        /// Name of session
        #[arg(short, long, default_value = "scratch")]
        name: String,
        #[clap(flatten)]
        scratch_args: ScratchDirArgs,
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
    #[clap(visible_alias = "ls")]
    List {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        filter_args: FilterArgs,
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Json)]
        output: OutputFormat,
        /// Only show specific field value.
        #[arg(long)]
        only: Option<OnlyOptions>,
    },
    /// List all tags used on projects tracked in your config list.
    ListTags {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Json)]
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
}

#[derive(ValueEnum, Debug, Clone)]
pub enum OnlyOptions {
    /// only show name.
    Name,
    /// only show safe name.
    SafeName,
    /// only show remote.
    Remote,
}

impl ProjectSubcommand {
    #[instrument(skip(project_sub_cmd, _context), err)]
    pub fn handle_cmd(project_sub_cmd: &Self, _context: &AxlContext) -> anyhow::Result<()> {
        match project_sub_cmd {
            Self::Open {
                proj_args,
                filter_args,
                sess_args,
                existing,
            } => {
                debug!(
                    "using [{:?}] projects file.",
                    proj_args.projects_config_path
                );
                if *existing {
                    trace!("picking from existing sessions...");
                    let sessions = sess_args.multiplexer.get_sessions()?;
                    sess_args
                        .multiplexer
                        .open_existing(&FzfCmd::pick_one_filtered(sessions)?)?;
                } else {
                    let projects_directory_file = ResolvedProjectDirectory::new_filtered(
                        &ConfigProjectDirectory::new(&proj_args.projects_config_path)?,
                        &filter_args.tags,
                    )?;
                    let project = projects_directory_file.get_project()?;
                    sess_args
                        .multiplexer
                        .open(&project.path, &project.safe_name)?;
                }
                Ok(())
            }
            Self::Scratch {
                proj_args: _,
                sess_args,
                name,
                scratch_args,
            } => {
                if sess_args.multiplexer.get_sessions()?.contains(name) {
                    let ans = Confirm::new(&format!(
                        "Opening existing session [{}], would you like to continue?",
                        &name
                    ))
                    .with_default(true)
                    .with_help_message(&format!("If you want to create a new session use --name flag, or kill the existing '{name}' session."))
                    .prompt()?;
                    if ans {
                        sess_args.multiplexer.open_existing(name)?;
                    }
                    return Ok(());
                }
                let dir = if scratch_args.home {
                    PathBuf::try_from(ConfigEnvKey::Home)?
                } else if let Some(proj_dir) = scratch_args.project_dir.clone() {
                    proj_dir
                } else {
                    let home = PathBuf::try_from(ConfigEnvKey::Home)?
                        .to_string_lossy()
                        .to_string();
                    PathBuf::from(
                        Text::new("Which path would you like to use?")
                            .with_validator(|input: &str| {
                                let path = PathBuf::from(input);
                                if path.exists() {
                                    Ok(Validation::Valid)
                                } else {
                                    Ok(Validation::Invalid(
                                        "Please enter a path that exists".into(),
                                    ))
                                }
                            })
                            .with_default(&home)
                            .with_initial_value(&home)
                            .prompt()?,
                    )
                };
                sess_args.multiplexer.open(&dir, name)?;
                Ok(())
            }
            Self::Kill { sess_args } => {
                let sessions = sess_args.multiplexer.get_sessions()?;
                debug!("sessions: {sessions:?}");
                let picked_sessions = FzfCmd::pick_many(sessions)?;
                let current_session = sess_args.multiplexer.get_current_session();
                debug!("current session: {current_session}");
                sess_args
                    .multiplexer
                    .kill_sessions(picked_sessions, &current_session)?;
                Ok(())
            }
            Self::Home { sess_args } => sess_args.multiplexer.unique_session(),
            Self::New { proj_args, ssh_uri } => {
                debug!(
                    "using [{:?}] projects file.",
                    proj_args.projects_config_path
                );
                let mut project_directory = ResolvedProjectDirectory::new(
                    &ConfigProjectDirectory::new(&proj_args.projects_config_path)?,
                )?;
                if project_directory
                    .projects
                    .iter()
                    .filter(|config_proj| &config_proj.remote == ssh_uri)
                    .count()
                    > 0
                {
                    eprintln!(
                        "{}",
                        "Project with this remote is already tracked.".red().bold()
                    );
                    return Ok(());
                }
                debug!("Attempting to clone {ssh_uri}...");
                let results =
                    GitRepo::from_url_multi(&[ssh_uri], &project_directory.projects_directory);
                for result in results {
                    if let Err(err) = result {
                        error!("Failed cloning with: {err:?}");
                    }
                }
                project_directory.add_config_projects(vec![ConfigProject {
                    name: None,
                    remote: ssh_uri.to_string(),
                    tags: BTreeSet::new(),
                }])?;
                println!("project was added to your root project_directory config.\nYou can now move it to a different group manually.");
                Ok(())
            }
            Self::Report {
                proj_args,
                filter_args,
            } => {
                let filtered_project_directory = ResolvedProjectDirectory::new_filtered(
                    &ConfigProjectDirectory::new(&proj_args.projects_config_path)?,
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
                    &proj_args.projects_config_path.to_string_lossy()
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
                    &ConfigProjectDirectory::new(&proj_args.projects_config_path)?,
                )?;

                let existing_projects = project_directory_file
                    .projects
                    .clone()
                    .into_iter()
                    .map(|ep| ep.remote)
                    .collect::<Vec<_>>();

                trace!("existing: {existing_projects:?}");

                let selected_projects = ResolvedProjectDirectory::pick_config_projects(
                    ResolvedProjectDirectory::get_projects_from_fs(directory)?
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
                only,
            } => {
                let filtered_project_directory = ResolvedProjectDirectory::new_filtered(
                    &ConfigProjectDirectory::new(&proj_args.projects_config_path)?,
                    &filter_args.tags,
                )?;
                let projects = filtered_project_directory.get_projects_from_remotes()?;
                if let Some(o) = only {
                    match o {
                        OnlyOptions::Name => {
                            formatted_print(
                                output,
                                projects.into_iter().map(|p| p.name).collect::<Vec<_>>(),
                            )?;
                        }
                        OnlyOptions::SafeName => {
                            formatted_print(
                                output,
                                projects
                                    .into_iter()
                                    .map(|p| p.safe_name)
                                    .collect::<Vec<_>>(),
                            )?;
                        }
                        OnlyOptions::Remote => {
                            formatted_print(
                                output,
                                projects.into_iter().map(|p| p.remote).collect::<Vec<_>>(),
                            )?;
                        }
                    }
                } else {
                    formatted_print(output, projects)?;
                }
                Ok(())
            }
            Self::ListTags { proj_args, output } => {
                let project_directory = ResolvedProjectDirectory::new(
                    &ConfigProjectDirectory::new(&proj_args.projects_config_path)?,
                )?;
                let tags = project_directory.get_projects_from_remotes()?.iter().fold(
                    BTreeSet::new(),
                    |mut acc, project| {
                        acc.extend(project.tags.clone());
                        acc
                    },
                );
                formatted_print(output, tags)
            }
        }
    }
}
