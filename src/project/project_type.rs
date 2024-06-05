use std::{
    collections::BTreeSet,
    fmt::Display,
    path::{Path, PathBuf},
};

use git_lib::{git::Git, git_uri::GitUri};
use serde::Serialize;
use serde_derive::Deserialize;
use tracing::instrument;

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ConfigProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub remote: String,
    #[serde(default = "tags_default", skip_serializing_if = "BTreeSet::is_empty")]
    pub tags: BTreeSet<String>,
}

const fn tags_default() -> BTreeSet<String> {
    BTreeSet::new()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedProject {
    pub project_folder_path: PathBuf,
    pub path: PathBuf,
    pub remote: String,
    pub git_uri: GitUri,
    pub tags: BTreeSet<String>,
}

impl ResolvedProject {
    #[instrument]
    pub fn new(path: &Path, remote: &str, tags: BTreeSet<String>) -> Result<Self, Error> {
        let git_uri = Git::parse_uri(remote).map_err(|_| Error::ProjectRemoteNotParsable)?;
        Ok(Self {
            project_folder_path: path.to_path_buf(),
            path: path.join(git_uri.name.clone()),
            remote: remote.to_string(),
            git_uri,
            tags,
        })
    }

    pub fn get_repo_uri(&self) -> String {
        format!(
            "https://{}/{}",
            self.git_uri.host.clone().expect("remote to have a host"),
            self.git_uri.fullname.clone(),
        )
    }

    pub fn get_name(&self) -> String {
        self.git_uri.name.clone()
    }

    pub fn get_remote(&self) -> String {
        self.remote.clone()
    }

    pub fn get_safe_name(&self) -> String {
        self.git_uri.name.replace('.', "_")
    }
}

impl Display for ResolvedProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.git_uri.name)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, path::PathBuf};

    use anyhow::Result;
    use git_lib::git::Git;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    use crate::project::project_type::ResolvedProject;

    #[rstest]
    fn should_create_new_project_with_no_tags() -> Result<()> {
        // Arrange
        let git_uri =
            Git::parse_uri("git@github.com:user/test2.git").expect("should parse successfully");

        // Act
        let project = ResolvedProject::new(
            &PathBuf::from("/test/projects/dir/"),
            "git@github.com:user/test2.git",
            BTreeSet::new(),
        )?;

        // Assert
        assert_eq!(
            project,
            ResolvedProject {
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string(),
                git_uri,
                tags: BTreeSet::new(),
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_create_new_project_with_tags() -> Result<()> {
        // Arrange
        let git_uri =
            Git::parse_uri("git@github.com:user/test2.git").expect("should parse successfully");

        // Act
        let project = ResolvedProject::new(
            &PathBuf::from("/test/projects/dir/"),
            "git@github.com:user/test2.git",
            BTreeSet::from_iter(vec!["tester".to_string(), "awesome_repo".to_string()]),
        )?;

        // Assert
        assert_eq!(
            project,
            ResolvedProject {
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string(),
                git_uri,
                tags: BTreeSet::from_iter(vec!["tester".to_string(), "awesome_repo".to_string()]),
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_handle_dot_at_start() -> Result<()> {
        // Arrange
        let git_uri =
            Git::parse_uri("git@github.com:user/.test2.git").expect("should parse successfully");

        // Act
        let project = ResolvedProject::new(
            &PathBuf::from("/test/projects/dir/"),
            "git@github.com:user/.test2.git",
            BTreeSet::from_iter(vec!["tester".to_string()]),
        )?;

        // Assert
        assert_eq!(
            project,
            ResolvedProject {
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/.test2".into(),
                remote: "git@github.com:user/.test2.git".to_string(),
                git_uri,
                tags: BTreeSet::from_iter(vec!["tester".to_string()]),
            }
        );
        assert_eq!(project.get_safe_name(), "_test2");

        Ok(())
    }
}
