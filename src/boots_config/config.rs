use serde::{Serialize, Deserialize};

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct BootsConfig {
    pub version: String,
    pub project_name: String,
    pub spec: ProjectTypes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum ProjectTypes {
    #[serde(rename_all = "kebab-case")]
    Npm {
        npm_version: Option<String>,
    },
    #[serde(rename_all = "kebab-case")]
    Yarn {},
}

impl ProjectTypes {
    fn get_allowed_targets(project_type: ProjectTypes) -> Vec<ArtifactTargets>{
        match project_type {
            ProjectTypes::Npm {..} => vec![ArtifactTargets::Image, ArtifactTargets::Tarball],
            ProjectTypes::Yarn {..} => vec![ArtifactTargets::Image],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "target_type")]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactTargets {
    Image,
    Tarball,
}
