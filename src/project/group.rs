use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::debug;

use serde::{Deserialize, Serialize};

use super::project_directory_manager::{ProjectConfigType, ProjectsDirectoryFile};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectGroupFile {
    #[serde(skip)]
    pub file_path: PathBuf,
    pub tags: Vec<String>,
    pub include: Vec<GroupItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum GroupItem {
    GroupFile(PathBuf),
    Project(ProjectConfigType)
}

impl ProjectGroupFile {
    pub fn new(path: &Path) -> Result<Self> {
        debug!("reading projects directory file...");
        let mut project_group_file: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        project_group_file.file_path = path.to_path_buf();
        debug!("finished reading project group file");
        Ok(project_group_file)
    }

    pub fn resolve_groups() -> Result<ProjectsDirectoryFile> {
        todo!("turn groups into ProjectsDirectoryFile")
    }

    pub fn recurse_group_files() -> Result<Vec<ProjectConfigType>> {
        Ok(vec![])
    }
}
