use anyhow::Result;
use colored::Colorize;
use git_lib::git::Git;
use std::path::{Path, PathBuf};

use crate::{config::ProjectConfig, fzf::FzfCmd, helper::get_directories, project::Project};

pub struct ProjectManager {
    pub root_dir: PathBuf,
    pub config: ProjectConfig,
}

impl ProjectManager {
    pub fn new(projects_dir: &Path, config: ProjectConfig) -> Self {
        Self {
            root_dir: projects_dir.to_path_buf(),
            config,
        }
    }

    pub fn get_project(
        &self,
        project_dir: &Option<PathBuf>,
        name: Option<String>,
    ) -> Result<Project> {
        project_dir.as_ref().map_or_else(
            || self.pick_project(),
            |selected_project| {
                Ok(Project::new(
                    selected_project,
                    name.unwrap_or_else(|| {
                        selected_project
                            .file_name()
                            .expect("selected project should have a valid file / dir name.")
                            .to_string_lossy()
                            .to_string()
                    }),
                    None,
                ))
            },
        )
    }

    pub fn get_projects_from_config(&self) -> Result<Vec<Project>> {
        self.config.project_folders.clone().map_or_else(
            || {
                panic!();
            },
            |projects| {
                let p: Vec<_> = projects
                    .into_iter()
                    .filter(|proj_folder| proj_folder.path == self.root_dir)
                    .flat_map(|d| {
                        d.projects
                            .into_iter()
                            .map(|config_project| {
                                Project::new(
                                    &d.path,
                                    Git::parse_url(&config_project.remote)
                                        .expect("provided git urls should be parsable")
                                        .name,
                                    Some(config_project.remote),
                                )
                            })
                            .collect::<Vec<Project>>()
                    })
                    .collect();
                Ok(p)
            },
        )
    }

    pub fn get_projects_from_fs(&self) -> Result<Vec<Project>> {
        let projects: Vec<_> = get_directories(&self.root_dir)?
            .into_iter()
            .map(|d| {
                Project::new(
                    &d,
                    d.file_name()
                        .expect("file_name should be representable as a String")
                        .to_string_lossy()
                        .to_string(),
                    Git::get_remote_url(&d).expect("git command to get remote should not fail"),
                )
            })
            .collect();
        Ok(projects)
    }

    pub fn pick_project(&self) -> Result<Project> {
        log::debug!("Using project_dir: {:?}", self.root_dir);

        let projects: Vec<_> = self.get_projects_from_config()?;
        let project_names = projects.iter().map(|p| p.name.clone()).collect::<Vec<_>>();

        log::debug!("projects: {projects:#?}");

        let project_name = FzfCmd::new().find_vec(project_names)?;

        projects
            .iter()
            .find(|p| p.name == project_name)
            .map_or_else(
                || {
                    eprintln!("{}", "No project was selected.".red().bold());
                    std::process::exit(1);
                },
                |project| Ok(project.clone()),
            )
    }
}
