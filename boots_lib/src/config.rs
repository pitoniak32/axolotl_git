use anyhow::Result;
use std::{collections::HashMap, fs, vec};

use serde::{Deserialize, Serialize};

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BootsConfig {
    pub project_name: String,
    pub workflow_config: Option<WorkflowConfig>,
    pub workflow_config_path: Option<String>,
    #[serde(flatten)]
    pub project_spec: ProjectOptions,
}

impl BootsConfig {
    pub fn new(path: &str) -> Result<Self> {
        let mut boots_config: BootsConfig = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        log::debug!("boots_config_before: {:#?}", &boots_config);
        let mut wf_config = None;
        if let Some(wf_path) = boots_config.workflow_config_path.clone() {
            wf_config = Some(serde_yaml::from_str::<WorkflowConfig>(
                &fs::read_to_string(wf_path)?,
            )?);
        };
        boots_config.workflow_config =
            WorkflowConfig::merge(boots_config.workflow_config, wf_config);
        log::debug!("boots_config_after: {:#?}", &boots_config);
        Ok(boots_config)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectOptions {
    package_manager_version: Option<String>,
    #[serde(default)]
    artifact_targets: Vec<ArtifactTargets>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "target-type")]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactTargets {
    #[serde(rename_all = "kebab-case")]
    Image { tagging_config: TaggingConfig },
    #[serde(rename_all = "kebab-case")]
    Tarball,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TaggingConfig {
    pub tags: Vec<LegalTags>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum LegalTags {
    Latest,
    BuildId,
    CommitHash,
    Major,
    MajorBuildId,
    MajorCommitHash,
    Minor,
    Patch,
    MajorMinor,
    MajorMinorBuildId,
    MajorMinorCommitHash,
    MajorMinorPatch,
    MajorMinorPatchBuildId,
    MajorMinorPatchCommitHash,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct WorkflowConfig {
    #[serde(default)]
    pub steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl WorkflowConfig {
    pub fn merge(
        wf_config_inline: Option<WorkflowConfig>,
        wf_config_file: Option<WorkflowConfig>,
    ) -> Option<Self> {
        match (wf_config_inline, wf_config_file) {
            (None, None) => None,
            (None, Some(wf_file)) => Some(wf_file),
            (Some(wf_inline), None) => Some(wf_inline),
            (Some(wf_inline), Some(wf_file)) => {
                // Read the config file
                let mut merged = WorkflowConfig {
                    steps: wf_inline.steps.iter().fold(
                        wf_file.steps.clone(),
                        |mut acc: Vec<WorkflowStep>, v| {
                            for a in acc.iter_mut() {
                                if a.step_type == v.step_type {
                                    a.env.extend(v.env.clone())
                                }
                            }
                            acc
                        },
                    ),
                    env: wf_file.env,
                };
                merged.env.extend(
                    wf_inline
                        .clone()
                        .env
                        .into_iter()
                        .map(|(k, v)| (k.clone(), v.clone())),
                );
                Some(merged)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct WorkflowStep {
    #[serde(rename = "type")]
    pub step_type: WorkflowStepType,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowStepType {
    Package,
    Lint,
    Test,
    Publish,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_merge_when_inline_none_and_file_none() {
        assert!(WorkflowConfig::merge(None, None).is_none());
    }

    #[test]
    fn should_merge_when_inline_some_and_file_none() {
        let expected = Some(WorkflowConfig {
            steps: vec![],
            env: HashMap::new(),
        });
        assert_eq!(
            WorkflowConfig::merge(
                Some(WorkflowConfig {
                    steps: vec![],
                    env: HashMap::new()
                }),
                None
            ),
            expected
        )
    }

    #[test]
    fn should_merge_when_inline_none_and_file_some() {
        let expected = Some(WorkflowConfig {
            steps: vec![],
            env: HashMap::new(),
        });
        assert_eq!(
            WorkflowConfig::merge(
                None,
                Some(WorkflowConfig {
                    steps: vec![],
                    env: HashMap::new()
                }),
            ),
            expected
        )
    }

    #[test]
    fn should_merge_when_inline_some_and_file_some() {
        let expected = Some(WorkflowConfig {
            steps: vec![
                WorkflowStep {
                    step_type: WorkflowStepType::Package,
                    env: HashMap::from([("RUST_LOG".to_string(), "trace".to_string())]),
                },
                WorkflowStep {
                    step_type: WorkflowStepType::Lint,
                    env: HashMap::new(),
                },
            ],
            env: HashMap::from([
                ("RUST_LOG".to_string(), "debug".to_string()),
                ("TEST".to_string(), "true".to_string()),
            ]),
        });

        assert_eq!(
            WorkflowConfig::merge(
                Some(WorkflowConfig {
                    steps: vec![WorkflowStep {
                        step_type: WorkflowStepType::Package,
                        env: HashMap::from([("RUST_LOG".to_string(), "trace".to_string())]),
                    }],
                    env: HashMap::from([("RUST_LOG".to_string(), "debug".to_string()),]),
                }),
                Some(WorkflowConfig {
                    steps: vec![
                        WorkflowStep {
                            step_type: WorkflowStepType::Package,
                            env: HashMap::from([("RUST_LOG".to_string(), "error".to_string())]),
                        },
                        WorkflowStep {
                            step_type: WorkflowStepType::Lint,
                            env: HashMap::new(),
                        }
                    ],
                    env: HashMap::from([
                        ("RUST_LOG".to_string(), "info".to_string()),
                        ("TEST".to_string(), "true".to_string())
                    ]),
                }),
            ),
            expected
        )
    }
}
