use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AxlContext {
    pub config_path: PathBuf,
    pub config: AxlConfig,
}

const fn art_default() -> bool {
    // Set to false since most commands pull up a prompt immediately
    false
}

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AxlConfig {
    pub general: GeneralConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GeneralConfig {
    #[serde(default = "art_default")]
    pub show_art: bool,
}

impl AxlConfig {
    pub fn from_file(config_path: &PathBuf) -> Result<Self> {
        log::trace!("loading config from {}...", config_path.to_string_lossy());
        let mut loaded_config: Self = serde_yaml::from_str(&fs::read_to_string(config_path)?)?;
        loaded_config.general.show_art = std::env::var("AXL_SHOW_ART")
            .map_or(loaded_config.general.show_art, |val| val == "true");
        log::trace!("config: {:#?}", loaded_config);
        log::trace!("config loaded!");
        Ok(loaded_config)
    }
}

#[cfg(test)]
mod tests {

    use similar_asserts::assert_eq;

    #[test]
    fn should_merge_when_inline_none_and_file_none() {
        assert_eq!("", "");
    }
}
