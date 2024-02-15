use anyhow::Result;
use colored::Colorize;
use git_lib::git::Git;
use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};

use crate::{fzf::FzfCmd, helper::get_directories, subcommand_project::Project};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectsDirectoryFile {
    pub path: PathBuf,
    pub projects: Vec<String>,
}

impl ProjectsDirectoryFile {
    pub fn new(path: &Path) -> Result<Self> {
        let projects_directory_file: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        Ok(projects_directory_file)
    }

    pub fn get_project(&self) -> Result<Project> {
        Self::pick_project(self.get_projects_from_remotes()?)
    }

    pub fn get_projects_from_remotes(&self) -> Result<Vec<Project>> {
        let p: Vec<_> = self
            .projects
            .iter()
            .map(|remote| {
                let name = Git::parse_url(remote)
                    .expect("provided git urls should be parsable")
                    .name;
                Project::new(&self.path, name, remote.to_string())
            })
            .collect();
        Ok(p)
    }

    pub fn get_projects_from_fs(path: &Path) -> Result<(Vec<Project>, Vec<PathBuf>)> {
        let mut ignored = vec![];
        let projects: Vec<_> = get_directories(path)?
            .into_iter()
            .filter_map(|d| {
                Git::get_remote_url(&d)
                    .expect("git command to get remote should not fail")
                    .map_or_else(
                        || {
                            log::warn!("skipping [{d:?}]. Remote was not found.");
                            ignored.push(d.clone());
                            None
                        },
                        |remote| {
                            Some(Project::new(
                                &d,
                                d.file_name()
                                    .expect("file_name should be representable as a String")
                                    .to_string_lossy()
                                    .to_string(),
                                remote,
                            ))
                        },
                    )
            })
            .collect();
        Ok((projects, ignored))
    }

    pub fn pick_project(projects: Vec<Project>) -> Result<Project> {
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

    pub fn pick_projects(pickable_projects: Vec<Project>) -> Result<Vec<Project>> {
        let project_names = pickable_projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>();

        log::debug!("pickable_projects: {pickable_projects:#?}");
        let project_names_picked = FzfCmd::new()
            .args(vec!["--phony", "--multi"])
            .find_vec(project_names)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        log::debug!("picked_project_names: {project_names_picked:?}");

        let projects = pickable_projects
            .into_iter()
            .filter(|p| project_names_picked.contains(&p.name))
            .collect::<Vec<_>>();

        if projects.is_empty() {
            eprintln!("{}", "No projects were selected.".red().bold());
            std::process::exit(1);
        }

        Ok(projects)
    }
}

pub struct ProjectDirectoryManager {
    pub projects_file: ProjectsDirectoryFile,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use assert_fs::{prelude::FileWriteStr, NamedTempFile};
    use rstest::{fixture, rstest};
    use similar_asserts::assert_eq;

    use crate::subcommand_project::Project;

    use super::ProjectsDirectoryFile;

    #[fixture]
    fn projects_directory_file_1() -> NamedTempFile {
        // Arrange
        let file = NamedTempFile::new("projects_test_1.txt")
            .expect("test fixture tmp file can be created");
        file.write_str("path: \"/test/projects/dir\"\nprojects:\n- git@github.com:user/test1.git\n- git@github.com:user/test2.git").expect("test fixture tmp file can be written to");
        file
    }

    #[fixture]
    fn projects_vec_len_2() -> Vec<Project> {
        vec![
            Project {
                name: "test1".to_string(),
                safe_name: "test1".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test1".into(),
                remote: "git@github.com:user/test1.git".to_string(),
            },
            Project {
                name: "test2".to_string(),
                safe_name: "test2".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string(),
            },
        ]
    }

    #[rstest]
    fn should_read_projects_file_into_struct(
        #[from(projects_directory_file_1)] test_file: NamedTempFile,
    ) -> Result<()> {
        // Act
        let projects_directory_file = ProjectsDirectoryFile::new(test_file.path())?;

        // Assert
        assert_eq!(
            projects_directory_file,
            ProjectsDirectoryFile {
                path: "/test/projects/dir".into(),
                projects: vec![
                    "git@github.com:user/test1.git".to_string(),
                    "git@github.com:user/test2.git".to_string()
                ]
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_turn_remotes_into_project_structs(
        #[from(projects_directory_file_1)] test_file: NamedTempFile,
    ) -> Result<()> {
        // Arrange
        let projects_directory_file = ProjectsDirectoryFile::new(test_file.path())?;

        // Act
        let projects = projects_directory_file.get_projects_from_remotes()?;

        // Assert
        assert_eq!(
            projects,
            vec![
                Project {
                    name: "test1".to_string(),
                    safe_name: "test1".to_string(),
                    project_folder_path: "/test/projects/dir/".into(),
                    path: "/test/projects/dir/test1".into(),
                    remote: "git@github.com:user/test1.git".to_string()
                },
                Project {
                    name: "test2".to_string(),
                    safe_name: "test2".to_string(),
                    project_folder_path: "/test/projects/dir/".into(),
                    path: "/test/projects/dir/test2".into(),
                    remote: "git@github.com:user/test2.git".to_string()
                },
            ]
        );

        Ok(())
    }
}
