use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const fn art_default() -> bool {
    true
}

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AxlConfig {
    #[serde(default = "art_default")]
    pub show_art: bool,
}

impl AxlConfig {
    pub fn new(path: &str) -> Result<Self> {
        let mut axl_config: Self = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        log::debug!("axl_config_before: {:#?}", &axl_config);
        let axl_inline = Self {
            show_art: std::env::var("AXL_SHOW_ART")
                .map_or(axl_config.show_art, |val| val == "true"),
        };
        axl_config = Self::merge(axl_inline, axl_config);
        log::debug!("axl_config_after: {:#?}", &axl_config);
        Ok(axl_config)
    }

    pub fn merge(axl_config_inline: Self, axl_config_file: Self) -> Self {
        log::debug!("merging configs...");
        Self {
            show_art: if !axl_config_inline.show_art {
                false
            } else {
                axl_config_file.show_art
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MukdukConfig {
    pub projects_dir: ProjectsDir,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProjectsDir {
    pub default: Option<PathBuf>,
    pub options: Option<Vec<PathBuf>>,
}

impl MukdukConfig {
    pub fn from_file(config_path: &PathBuf) -> Result<Self> {
        log::trace!("loading config from {}...", config_path.to_string_lossy());
        let loaded_config = toml::from_str(&fs::read_to_string(config_path)?)?;
        log::trace!("config: {:#?}", loaded_config);
        log::trace!("config loaded!");
        Ok(loaded_config)
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    #[test]
    fn should_merge_when_inline_none_and_file_none() {
        assert_eq!("", "");
    }
}
