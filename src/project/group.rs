use anyhow::Result;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use tracing::debug;

use serde::{Deserialize, Serialize};

use super::project_type::ConfigProject;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ProjectGroupFile {
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(default = "tags_default")]
    pub tags: BTreeSet<String>,
    pub include: Vec<GroupItem>,
}

const fn tags_default() -> BTreeSet<String> {
    BTreeSet::new()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum GroupItem {
    GroupFile(PathBuf),
    Project(ConfigProject),
}

impl ProjectGroupFile {
    pub fn new(path: &Path) -> Result<Self> {
        debug!("reading projects directory file...");
        let mut project_group_file: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        project_group_file.file_path = path.to_path_buf();
        debug!("finished reading project group file");
        Ok(project_group_file)
    }

    pub fn get_projects(&self) -> Result<Vec<ConfigProject>> {
        self.recurse_group_files(BTreeSet::new())
    }

    fn recurse_group_files(&self, tags: BTreeSet<String>) -> Result<Vec<ConfigProject>> {
        let mut projects: Vec<_> = vec![];
        let mut group_tags = tags;
        group_tags.extend(self.tags.clone());
        for item in self.include.clone() {
            match item {
                GroupItem::GroupFile(group_path) => {
                    let config_projects = Self::new(&group_path)?
                        .recurse_group_files(group_tags.clone())?
                        .iter_mut()
                        .map(|p| {
                            p.tags.extend(group_tags.clone());
                            p.clone()
                        })
                        .collect::<Vec<_>>();
                    projects.extend(config_projects);
                }
                GroupItem::Project(mut p) => {
                    p.tags.extend(group_tags.clone());
                    projects.push(p)
                }
            }
        }
        Ok(projects)
    }
}
