use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const fn art_default() -> bool {
    // Set to false since most commands pull up a prompt immediately
    false
}

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AxlConfig {
    #[serde(default = "art_default")]
    pub show_art: bool,

    pub projects_dir: ProjectsDir,
}

impl AxlConfig {
    pub fn from_file(config_path: &PathBuf) -> Result<Self> {
        log::trace!("loading config from {}...", config_path.to_string_lossy());
        let mut loaded_config: Self = toml::from_str(&fs::read_to_string(config_path)?)?;
        loaded_config.show_art =
            std::env::var("AXL_SHOW_ART").map_or(loaded_config.show_art, |val| val == "true");
        log::trace!("config: {:#?}", loaded_config);
        log::trace!("config loaded!");
        Ok(loaded_config)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProjectsDir {
    pub default: Option<PathBuf>,
    pub options: Option<Vec<PathBuf>>,
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    #[test]
    fn should_merge_when_inline_none_and_file_none() {
        assert_eq!("", "");
    }
}
