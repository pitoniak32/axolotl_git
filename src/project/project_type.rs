use std::{
    collections::BTreeSet,
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_derive::Deserialize;
use tracing::instrument;

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
    pub name: String,
    pub safe_name: String,
    pub remote: String,
    pub tags: BTreeSet<String>,
}

impl ResolvedProject {
    #[instrument]
    pub fn new(path: &Path, name: String, remote: String, tags: BTreeSet<String>) -> Self {
        Self {
            project_folder_path: path.to_path_buf(),
            path: path.join(name.clone()),
            name: name.clone(),
            safe_name: name.replace('.', "_"),
            remote,
            tags,
        }
    }
}

impl Display for ResolvedProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, path::PathBuf};

    use anyhow::Result;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    use crate::project::project_type::ResolvedProject;

    #[rstest]
    fn should_create_new_project_with_no_tags() -> Result<()> {
        // Act
        let project = ResolvedProject::new(
            &PathBuf::from("/test/projects/dir/"),
            "test2".to_string(),
            "git@github.com:user/test2.git".to_string(),
            BTreeSet::new(),
        );

        // Assert
        assert_eq!(
            project,
            ResolvedProject {
                name: "test2".to_string(),
                safe_name: "test2".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string(),
                tags: BTreeSet::new(),
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_create_new_project_with_tags() -> Result<()> {
        // Act
        let project = ResolvedProject::new(
            &PathBuf::from("/test/projects/dir/"),
            "test2".to_string(),
            "git@github.com:user/test2.git".to_string(),
            BTreeSet::from_iter(vec!["tester".to_string(), "awesome_repo".to_string()]),
        );

        // Assert
        assert_eq!(
            project,
            ResolvedProject {
                name: "test2".to_string(),
                safe_name: "test2".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/test2".into(),
                remote: "git@github.com:user/test2.git".to_string(),
                tags: BTreeSet::from_iter(vec!["tester".to_string(), "awesome_repo".to_string()]),
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_handle_dot_at_start() -> Result<()> {
        // Act
        let project = ResolvedProject::new(
            &PathBuf::from("/test/projects/dir/"),
            ".test2".to_string(),
            "git@github.com:user/.test2.git".to_string(),
            BTreeSet::from_iter(vec!["tester".to_string()]),
        );

        // Assert
        assert_eq!(
            project,
            ResolvedProject {
                name: ".test2".to_string(),
                safe_name: "_test2".to_string(),
                project_folder_path: "/test/projects/dir/".into(),
                path: "/test/projects/dir/.test2".into(),
                remote: "git@github.com:user/.test2.git".to_string(),
                tags: BTreeSet::from_iter(vec!["tester".to_string()]),
            }
        );

        Ok(())
    }
}
