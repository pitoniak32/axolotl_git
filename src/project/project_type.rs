use std::{
    collections::BTreeSet,
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Project {
    pub project_folder_path: PathBuf,
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,
    pub remote: String,
    pub tags: BTreeSet<String>,
}

impl Project {
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

    #[instrument]
    pub fn get_safe_name(&self) -> String {
        self.safe_name.clone()
    }

    #[instrument]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    #[instrument]
    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Display for Project {
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

    use crate::project::project_type::Project;

    #[rstest]
    fn should_create_new_project_with_no_tags() -> Result<()> {
        // Act
        let project = Project::new(
            &PathBuf::from("/test/projects/dir/"),
            "test2".to_string(),
            "git@github.com:user/test2.git".to_string(),
            BTreeSet::new(),
        );

        // Assert
        assert_eq!(
            project,
            Project {
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
        let project = Project::new(
            &PathBuf::from("/test/projects/dir/"),
            "test2".to_string(),
            "git@github.com:user/test2.git".to_string(),
            BTreeSet::from_iter(vec!["tester".to_string(), "awesome_repo".to_string()]),
        );

        // Assert
        assert_eq!(
            project,
            Project {
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
        let project = Project::new(
            &PathBuf::from("/test/projects/dir/"),
            ".test2".to_string(),
            "git@github.com:user/.test2.git".to_string(),
            BTreeSet::from_iter(vec!["tester".to_string()]),
        );

        // Assert
        assert_eq!(
            project,
            Project {
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
