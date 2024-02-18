use anyhow::Result;
use colored::Colorize;
use git_lib::git::Git;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, instrument, warn};

use serde_derive::{Deserialize, Serialize};

use crate::{error::AxlError, fzf::FzfCmd, helper::get_directories};

use super::project_type::Project;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectsDirectoryFile {
    pub path: PathBuf,
    pub projects: Vec<ProjectConfigType>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ProjectConfigType {
    name: Option<String>,
    remote: String,
    tags: Option<Vec<String>>,
}

impl ProjectsDirectoryFile {
    #[instrument(err)]
    pub fn new(path: &Path) -> Result<Self> {
        let projects_directory_file: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        Ok(projects_directory_file)
    }

    #[instrument(err)]
    pub fn new_filtered(path: &Path, tags: &Option<Vec<String>>) -> Result<Self> {
        let mut projects_directory_file: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;

        if let Some(given_tags) = tags {
            let filtered = projects_directory_file
                .projects
                .clone()
                .into_iter()
                .filter(|project| {
                    project.tags.clone().is_some_and(|project_tags| {
                        project_tags.iter().any(|tag| given_tags.contains(tag))
                    })
                })
                .collect::<Vec<_>>();

            if filtered.len() < projects_directory_file.projects.len() {
                projects_directory_file.projects = filtered;
            }
        }

        Ok(projects_directory_file)
    }

    #[instrument(err)]
    pub fn get_project(&self) -> Result<Project> {
        Self::pick_project(self.get_projects_from_remotes()?)
    }

    #[instrument(err)]
    pub fn get_projects_from_remotes(&self) -> Result<Vec<Project>> {
        self.projects
            .iter()
            .map(|project_config_type| {
                Ok(Project::new(
                    &self.path,
                    project_config_type
                        .name
                        .clone()
                        .unwrap_or(Git::parse_url(&project_config_type.remote)?.name),
                    project_config_type.remote.to_string(),
                    project_config_type.tags.clone(),
                ))
            })
            .collect::<Result<Vec<Project>>>()
    }

    #[instrument(err)]
    pub fn get_projects_from_fs(path: &Path) -> Result<(Vec<Project>, Vec<PathBuf>)> {
        let mut ignored = vec![];
        let projects: Vec<_> = get_directories(path)?
            .into_iter()
            .filter_map(|d| {
                Git::get_remote_url("origin", &d)
                    .expect("git command to get remote should not fail")
                    .map_or_else(
                        || {
                            warn!("skipping [{d:?}]. Remote was not found.");
                            ignored.push(d.clone());
                            None
                        },
                        |remote| {
                            Some(Project::new(
                                path,
                                d.file_name()
                                    .expect("file_name should be representable as a String")
                                    .to_string_lossy()
                                    .to_string(),
                                remote,
                                None,
                            ))
                        },
                    )
            })
            .collect();
        Ok((projects, ignored))
    }

    #[instrument(err)]
    pub fn pick_project(projects: Vec<Project>) -> Result<Project> {
        let project_names = projects.iter().map(|p| p.name.clone()).collect::<Vec<_>>();

        let project_name = FzfCmd::new().find_vec(project_names)?;

        projects
            .iter()
            .find(|p| p.name == project_name)
            .map_or_else(
                || {
                    eprintln!("{}", "No project was selected.".red().bold());
                    Err(AxlError::NoProjectSelected)?
                },
                |project| Ok(project.clone()),
            )
    }

    #[instrument(err)]
    pub fn pick_projects(pickable_projects: Vec<Project>) -> Result<Vec<Project>> {
        let project_names = pickable_projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>();

        let project_names_picked = FzfCmd::new()
            .args(vec!["--phony", "--multi"])
            .find_vec(project_names)?
            .trim_end()
            .split('\n')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        debug!("picked_project_names: {project_names_picked:?}");

        let projects = pickable_projects
            .into_iter()
            .filter(|p| project_names_picked.contains(&p.name))
            .collect::<Vec<_>>();

        if projects.is_empty() {
            eprintln!("{}", "No projects were selected.".red().bold());
            Err(AxlError::NoProjectSelected)?
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

    use assert_fs::prelude::*;
    use assert_fs::*;
    use git_lib::git::Git;
    use rstest::{fixture, rstest};
    use similar_asserts::assert_eq;

    use crate::project::{project_directory_manager::ProjectConfigType, project_type::Project};

    use super::ProjectsDirectoryFile;

    #[fixture]
    fn projects_directory_file_1() -> NamedTempFile {
        // Arrange
        let file = NamedTempFile::new("projects_test_1.txt")
            .expect("test fixture tmp file can be created");
        file.write_str(
            "path: \"/test/projects/dir\"
projects:
  - remote: git@github.com:user/test1.git
    tags:
      - tester_repo
      - prod
  - remote: git@github.com:user/test2.git
    tags: [grouped]
    name: test2_rename",
        )
        .expect("test fixture tmp file can be written to");
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
                tags: Some(vec!["test1".to_string()]),
            },
            Project {
                name: "test2".to_string(),
                safe_name: "test2".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string(),
                tags: None,
            },
        ]
    }

    #[fixture]
    fn projects_directory_fs() -> TempDir {
        // Arrange
        let projects = TempDir::new().expect("should be able to make temp dir");

        let child_config = projects.child("project_config.yml");
        child_config
            .touch()
            .expect("child_config should get created");
        child_config
            .write_str(&format!(
                "path: \"{}\"\nprojects:\n- remote: git@github.com:test_user/test_repo1.git\n- remote: git@github.com:test_user/test_repo2.git",
                &projects.path().join("projects").to_string_lossy()
            ))
            .expect("should be able to write to file");

        make_test_repo(&projects, "test_repo1");
        make_test_repo(&projects, "test_repo2");
        let child_repo3 = projects.child("projects/test_repo3_not_tracked/file");
        child_repo3
            .touch()
            .expect("should be able to create a file");

        projects
    }

    // make into partial fixture when not drunk
    fn make_test_repo(dir: &TempDir, name: &str) {
        let child_repo = dir.child(format!("projects/{name}/file"));
        let repo_dir = child_repo.parent().expect("should have parent");
        child_repo.touch().expect("child_repo should get created");
        Git::init(repo_dir).expect("child_repo can be initilized");
        Git::add_remote(
            "origin",
            &format!("git@github.com:test_user/{name}.git"),
            repo_dir,
        )
        .expect("child_repo can have remote added");
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
                    ProjectConfigType {
                        remote: "git@github.com:user/test1.git".to_string(),
                        name: None,
                        tags: Some(vec!["tester_repo".to_string(), "prod".to_string()])
                    },
                    ProjectConfigType {
                        remote: "git@github.com:user/test2.git".to_string(),
                        name: Some("test2_rename".to_string()),
                        tags: Some(vec!["grouped".to_string()])
                    },
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
                    remote: "git@github.com:user/test1.git".to_string(),
                    tags: Some(vec!["tester_repo".to_string(), "prod".to_string()]),
                },
                Project {
                    name: "test2_rename".to_string(),
                    safe_name: "test2_rename".to_string(),
                    project_folder_path: "/test/projects/dir/".into(),
                    path: "/test/projects/dir/test2_rename".into(),
                    remote: "git@github.com:user/test2.git".to_string(),
                    tags: Some(vec!["grouped".to_string()]),
                },
            ]
        );

        Ok(())
    }

    #[rstest]
    fn should_turn_remotes_into_project_structs_and_filter_by_tags(
        #[from(projects_directory_file_1)] test_file: NamedTempFile,
    ) -> Result<()> {
        // Arrange
        let projects_directory_file =
            ProjectsDirectoryFile::new_filtered(test_file.path(), &Some(vec!["prod".to_string()]))?;

        // Act
        let projects = projects_directory_file.get_projects_from_remotes()?;

        // Assert
        assert_eq!(
            projects,
            vec![Project {
                name: "test1".to_string(),
                safe_name: "test1".to_string(),
                project_folder_path: "/test/projects/dir".into(),
                path: "/test/projects/dir/test1".into(),
                remote: "git@github.com:user/test1.git".to_string(),
                tags: Some(vec!["tester_repo".to_string(), "prod".to_string()]),
            },]
        );

        Ok(())
    }

    #[rstest]
    fn should_read_projects_from_fs(
        #[from(projects_directory_fs)] test_dir: TempDir,
    ) -> Result<()> {
        // Act
        let projects_dir_path = test_dir.path().join("projects");
        dbg!(&projects_dir_path);
        let projects = ProjectsDirectoryFile::get_projects_from_fs(&projects_dir_path)?;

        assert_eq!(projects.0.len(), 2);
        assert!(projects.0.contains(&Project {
            name: "test_repo1".to_string(),
            safe_name: "test_repo1".to_string(),
            project_folder_path: projects_dir_path.clone(),
            path: projects_dir_path.join("test_repo1"),
            remote: "git@github.com:test_user/test_repo1.git".to_string(),
            tags: None,
        },),);
        assert!(projects.0.contains(&Project {
            name: "test_repo2".to_string(),
            safe_name: "test_repo2".to_string(),
            project_folder_path: projects_dir_path.clone(),
            path: projects_dir_path.join("test_repo2"),
            remote: "git@github.com:test_user/test_repo2.git".to_string(),
            tags: None,
        }));
        assert_eq!(
            projects.1,
            vec![projects_dir_path.join("test_repo3_not_tracked"),]
        );

        test_dir.close().expect("temp dir can be closed");

        Ok(())
    }
}
