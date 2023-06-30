use serde::{Serialize, Deserialize};

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct BootsConfig {
    pub boots_version: String,
    pub project_name: String,
    pub spec: ProjectTypes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum ProjectTypes {
    #[serde(rename_all = "kebab-case")]
    Rust {
        cargo_version: Option<String>,
        artifact_targets: Vec<ArtifactTargets>,
    },
    #[serde(rename_all = "kebab-case")]
    Npm {
        npm_version: Option<String>,
        artifact_targets: Vec<ArtifactTargets>,
    },
    #[serde(rename_all = "kebab-case")]
    Yarn {
        yarn_version: Option<String>,
        artifact_targets: Vec<ArtifactTargets>,
    },
    #[serde(rename_all = "kebab-case")]
    Go {
        make_version: Option<String>,
        artifact_targets: Vec<ArtifactTargets>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "target-type")]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactTargets {
    #[serde(rename_all = "kebab-case")]
    Image {
        tagging_config: TaggingConfig,
    },
    #[serde(rename_all = "kebab-case")]
    Tarball,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TaggingConfig {
    tags: Vec<LegalTags>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum LegalTags {
    Latest,
    BuildId,
    CommitHash,
    #[serde(rename = "major")]
    SemVerMajor,
    #[serde(rename = "minor")]
    SemVerMinor,
    #[serde(rename = "patch")]
    SemVerPatch,
    #[serde(rename = "major-minor")]
    SemVerMajorMinor,
    #[serde(rename = "major-minor-patch")]
    SemVerMajorMinorPatch,
}
