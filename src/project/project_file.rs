use anyhow::Result;
use console::Style;
use git_lib::git::Git;
use inquire::Confirm;
use similar::{ChangeTag, TextDiff};
use spinners::{Spinner, Spinners};
use std::{
    collections::BTreeSet,
    fs::{self},
    path::{Path, PathBuf},
};
use tracing::{instrument, trace, warn};

use serde_derive::{Deserialize, Serialize};

use crate::{error::Error, fzf::FzfCmd, helper::get_directories, project::group::ProjectGroupFile};

use super::{
    group::GroupItem,
    project_type::{ConfigProject, ResolvedProject},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigProjectDirectory {
    #[serde(skip)]
    pub file_path: PathBuf,
    pub projects_directory: PathBuf,
    pub include: Vec<GroupItem>,
}

impl ConfigProjectDirectory {
    #[instrument(err)]
    pub fn new(path: &Path) -> Result<Self> {
        trace!("reading projects directory file...");
        let mut projects_directory_file: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        projects_directory_file.file_path = path.to_path_buf();
        trace!("finished reading projects directory file");
        Ok(projects_directory_file)
    }

    #[instrument(err)]
    pub fn resolve_projects(&self) -> Result<Vec<ConfigProject>> {
        trace!("loading group files, and projects...");
        let mut projects = vec![];
        for item in self.include.clone() {
            match item {
                GroupItem::GroupFile(path) => {
                    let group_file = ProjectGroupFile::new(&path)?;
                    projects.extend(group_file.get_projects()?);
                }
                GroupItem::Project(p) => projects.push(p),
            };
        }
        trace!("finished loading group files, and projects");
        Ok(projects)
    }

    pub fn add_config_projects(&mut self, projects: Vec<ConfigProject>) -> Result<()> {
        for project in projects {
            self.include.push(GroupItem::Project(project));
        }
        Ok(())
    }

    #[instrument(err)]
    pub fn save_file(&self) -> Result<()> {
        fs::write(&self.file_path, serde_yaml::to_string::<Self>(self)?)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedProjectDirectory {
    pub resolved_from_path: PathBuf,
    pub projects_directory: PathBuf,
    pub projects: Vec<ConfigProject>,
}

impl ResolvedProjectDirectory {
    pub fn new(project_directory_file: &ConfigProjectDirectory) -> Result<Self> {
        Ok(Self {
            resolved_from_path: project_directory_file.file_path.clone(),
            projects_directory: project_directory_file.projects_directory.clone(),
            projects: project_directory_file.resolve_projects()?,
        })
    }

    #[instrument(err)]
    pub fn new_filtered(
        project_directory_file: &ConfigProjectDirectory,
        tags: &Vec<String>,
    ) -> Result<Self> {
        let mut project_directory: Self = Self::new(project_directory_file)?;

        if !tags.is_empty() {
            let filtered = project_directory
                .projects
                .clone()
                .into_iter()
                .filter(|project| project.tags.iter().any(|tag| tags.contains(tag)))
                .collect::<Vec<_>>();

            if filtered.len() < project_directory.projects.len() {
                project_directory.projects = filtered;
            }
        }

        Ok(project_directory)
    }

    #[instrument(err)]
    pub fn get_project(&self) -> Result<ResolvedProject> {
        Self::pick_project(self.get_projects_from_remotes()?)
    }

    #[instrument(err)]
    pub fn get_projects_from_remotes(&self) -> Result<Vec<ResolvedProject>> {
        self.projects
            .iter()
            .map(|project_config_type| {
                Ok(ResolvedProject::new(
                    &self.projects_directory,
                    &project_config_type.remote,
                    project_config_type.tags.clone(),
                )?)
            })
            .collect::<Result<Vec<ResolvedProject>>>()
    }

    #[instrument(err)]
    pub fn get_projects_from_fs(path: &Path) -> Result<(Vec<ResolvedProject>, Vec<PathBuf>)> {
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
                        |remote| ResolvedProject::new(path, &remote, BTreeSet::new()).ok(),
                    )
            })
            .collect();
        Ok((projects, ignored))
    }

    #[instrument(err)]
    pub fn pick_project(projects: Vec<ResolvedProject>) -> Result<ResolvedProject> {
        let project_names = projects.iter().map(|p| p.get_name()).collect::<Vec<_>>();

        let project_name = FzfCmd::new().find_vec(project_names)?;

        projects
            .iter()
            .find(|p| p.get_name() == project_name)
            .map_or_else(
                || Err(Error::NoProjectSelected)?,
                |project| Ok(project.clone()),
            )
    }

    #[instrument(err)]
    pub fn pick_projects(pickable_projects: Vec<ResolvedProject>) -> Result<Vec<ResolvedProject>> {
        let project_names = pickable_projects
            .iter()
            .map(|p| p.get_name())
            .collect::<Vec<_>>();

        let project_names_picked = FzfCmd::pick_many(project_names)?;

        let projects = pickable_projects
            .into_iter()
            .filter(|p| project_names_picked.contains(&p.get_name()))
            .collect::<Vec<_>>();

        if projects.is_empty() {
            Err(Error::NoProjectSelected)?
        }

        Ok(projects)
    }

    #[instrument(err)]
    pub fn pick_config_projects(
        pickable_projects: Vec<ConfigProject>,
    ) -> Result<Vec<ConfigProject>> {
        let project_remotes = pickable_projects
            .iter()
            .map(|p| p.remote.clone())
            .collect::<Vec<_>>();

        let project_remotes_picked = FzfCmd::pick_many(project_remotes)?;

        let projects = pickable_projects
            .into_iter()
            .filter(|p| project_remotes_picked.contains(&p.remote))
            .collect::<Vec<_>>();

        if projects.is_empty() {
            Err(Error::NoProjectSelected)?
        }

        Ok(projects)
    }

    pub fn add_config_projects(&self, projects: Vec<ConfigProject>) -> Result<()> {
        let mut config_project_directory = ConfigProjectDirectory::new(&self.resolved_from_path)?;
        let before = serde_yaml::to_string(&config_project_directory.resolve_projects()?)?;
        config_project_directory.add_config_projects(projects)?;
        let after = serde_yaml::to_string(&config_project_directory.resolve_projects()?)?;
        let diff = TextDiff::from_lines(&before, &after);
        println!("project file diff:\n----");
        for change in diff.iter_all_changes() {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
            };
            print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
        }

        let file_string = self.resolved_from_path.to_string_lossy();
        let ans = Confirm::new("Do you accept these changes?")
            .with_default(false)
            .with_help_message(&format!("These changes will be saved to [{file_string}]"))
            .prompt()?;

        if ans {
            let mut sp = Spinner::new(Spinners::Dots9, format!("Saving to {file_string}..."));
            config_project_directory.save_file()?;
            sp.stop_and_persist(
                &Style::new().green().apply_to("✓").bold().to_string(),
                "Saved".into(),
            );
        } else {
            println!("projects file will not be updated.")
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{collections::BTreeSet, fs};

    use anyhow::Result;

    use assert_fs::{fixture::ChildPath, prelude::*, *};
    use git_lib::git::Git;
    use rstest::{fixture, rstest};
    use similar_asserts::assert_eq;

    use crate::project::{
        project_file::ConfigProjectDirectory,
        project_type::{ConfigProject, ResolvedProject},
    };

    use super::ResolvedProjectDirectory;

    #[fixture]
    fn projects_directory_file_1() -> (TempDir, ChildPath, ChildPath) {
        // Arrange
        let dir = TempDir::new().expect("temp dir can be created");
        let file = dir.child("projects_test_1.yml");
        let group_file = dir.child("projects_group_1.yml");

        file.write_str(&format!(
            "projects_directory: \"/test/projects/dir\"
include:
  - {}
  - remote: git@github.com:user/test1.git
    tags:
      - tester_repo
      - prod
  - remote: git@github.com:user/test2.git
    tags: [grouped]
    name: test2_rename",
            group_file.path().to_string_lossy()
        ))
        .expect("test fixture tmp file can be written to");

        group_file
            .write_str(
                "tags: ['grouped']
include:
  - remote: git@github.com:user/test3.git
    tags: ['test3']",
            )
            .expect("test fixture tmp file can be written to");

        (dir, file, group_file)
    }

    #[fixture]
    fn projects_vec_len_2() -> Vec<ResolvedProject> {
        let remote1 = "git@github.com:user/test1.git".to_string();
        let remote2 = "git@github.com:user/test2.git".to_string();
        vec![
            ResolvedProject {
                git_uri: Git::parse_uri(&remote1).expect("to be valid uri"),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test1".into(),
                remote: remote1,
                tags: BTreeSet::from_iter(vec!["test1".to_string()]),
            },
            ResolvedProject {
                git_uri: Git::parse_uri(&remote2).expect("to be valid uri"),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: remote2,
                tags: BTreeSet::new(),
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
        #[from(projects_directory_file_1)] test_dir: (TempDir, ChildPath, ChildPath),
    ) -> Result<()> {
        // Arrange
        let project_1 = ConfigProject {
            remote: "git@github.com:user/test1.git".to_string(),
            name: None,
            tags: BTreeSet::from_iter(vec!["tester_repo".to_string(), "prod".to_string()]),
        };
        let project_2 = ConfigProject {
            remote: "git@github.com:user/test2.git".to_string(),
            name: Some("test2_rename".to_string()),
            tags: BTreeSet::from_iter(vec!["grouped".to_string()]),
        };
        let project_3 = ConfigProject {
            remote: "git@github.com:user/test3.git".to_string(),
            name: None,
            tags: BTreeSet::from_iter(vec!["grouped".to_string(), "test3".to_string()]),
        };

        // Act
        dbg!(test_dir.1.path());
        dbg!(fs::read_to_string(test_dir.1.path())?);
        let project_directory =
            ResolvedProjectDirectory::new(&ConfigProjectDirectory::new(test_dir.1.path())?)?;

        // Assert
        assert_eq!(
            project_directory,
            ResolvedProjectDirectory {
                resolved_from_path: test_dir.1.path().to_path_buf(),
                projects_directory: "/test/projects/dir".into(),
                projects: vec![project_3, project_1, project_2]
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_turn_remotes_into_project_structs(
        #[from(projects_directory_file_1)] test_dir: (TempDir, ChildPath, ChildPath),
    ) -> Result<()> {
        // Arrange
        let project_directory =
            ResolvedProjectDirectory::new(&ConfigProjectDirectory::new(test_dir.1.path())?)?;

        let remote1 = "git@github.com:user/test1.git".to_string();
        let remote2 = "git@github.com:user/test2.git".to_string();
        let remote3 = "git@github.com:user/test3.git".to_string();

        // Act
        let projects = project_directory.get_projects_from_remotes()?;

        // Assert
        assert_eq!(
            projects,
            vec![
                ResolvedProject {
                    project_folder_path: "/test/projects/dir".into(),
                    git_uri: Git::parse_uri(&remote3).expect("valid remote"),
                    path: "/test/projects/dir/test3".into(),
                    remote: remote3,
                    tags: BTreeSet::from_iter(vec!["grouped".to_string(), "test3".to_string()]),
                },
                ResolvedProject {
                    project_folder_path: "/test/projects/dir".into(),
                    git_uri: Git::parse_uri(&remote1).expect("valid remote"),
                    path: "/test/projects/dir/test1".into(),
                    remote: remote1,
                    tags: BTreeSet::from_iter(vec!["tester_repo".to_string(), "prod".to_string()]),
                },
                ResolvedProject {
                    project_folder_path: "/test/projects/dir".into(),
                    git_uri: Git::parse_uri(&remote2).expect("valid remote"),
                    path: "/test/projects/dir/test2".into(),
                    remote: remote2,
                    tags: BTreeSet::from_iter(vec!["grouped".to_string()]),
                },
            ]
        );

        Ok(())
    }

    #[rstest]
    fn should_turn_remotes_into_project_structs_and_filter_by_tags(
        #[from(projects_directory_file_1)] test_dir: (TempDir, ChildPath, ChildPath),
    ) -> Result<()> {
        // Arrange
        let project_directory = ResolvedProjectDirectory::new_filtered(
            &ConfigProjectDirectory::new(test_dir.1.path())?,
            &vec!["prod".to_string()],
        )?;
        let remote1 = "git@github.com:user/test1.git".to_string();

        // Act
        let projects = project_directory.get_projects_from_remotes()?;

        // Assert
        assert_eq!(
            projects,
            vec![ResolvedProject {
                project_folder_path: "/test/projects/dir".into(),
                git_uri: Git::parse_uri(&remote1).expect("valid remote"),
                path: "/test/projects/dir/test1".into(),
                remote: remote1,
                tags: BTreeSet::from_iter(vec!["tester_repo".to_string(), "prod".to_string()]),
            },]
        );

        Ok(())
    }

    #[rstest]
    fn should_read_projects_from_fs(
        #[from(projects_directory_fs)] test_dir: TempDir,
    ) -> Result<()> {
        // Arrange
        let remote1 = "git@github.com:test_user/test_repo1.git".to_string();
        let remote2 = "git@github.com:test_user/test_repo2.git".to_string();

        // Act
        let projects_dir_path = test_dir.path().join("projects");
        let projects = ResolvedProjectDirectory::get_projects_from_fs(&projects_dir_path)?;

        assert_eq!(projects.0.len(), 2);
        assert!(projects.0.contains(&ResolvedProject {
            project_folder_path: projects_dir_path.clone(),
            git_uri: Git::parse_uri(&remote1).expect("valid remote"),
            path: projects_dir_path.join("test_repo1"),
            remote: remote1,
            tags: BTreeSet::new(),
        },),);
        assert!(projects.0.contains(&ResolvedProject {
            project_folder_path: projects_dir_path.clone(),
            git_uri: Git::parse_uri(&remote2).expect("valid remote"),
            path: projects_dir_path.join("test_repo2"),
            remote: remote2,
            tags: BTreeSet::new(),
        }));
        assert_eq!(
            projects.1,
            vec![projects_dir_path.join("test_repo3_not_tracked"),]
        );

        test_dir.close().expect("temp dir can be closed");

        Ok(())
    }
}
