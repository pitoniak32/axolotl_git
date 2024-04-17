use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{info, instrument};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AxlContext {
    pub config_path: PathBuf,
    pub config: AxlConfig,
}

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct AxlConfig {
    pub general: GeneralConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GeneralConfig {
    #[serde(default = "art_default")]
    pub show_art: Option<bool>,
    #[serde(default = "version_default")]
    pub show_version: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            show_art: art_default(),
            show_version: version_default(),
        }
    }
}

const fn art_default() -> Option<bool> {
    // Set to false since most commands pull up a prompt immediately
    None
}

const fn version_default() -> bool {
    true
}

impl AxlConfig {
    #[instrument(err)]
    pub fn from_file(config_path: &Path) -> Result<Self> {
        let config_string = &fs::read_to_string(config_path)?;
        let mut loaded_config = if !config_string.trim().is_empty() {
            serde_yaml::from_str(config_string)?
        } else {
            let mut config = Self::default();
            config.general.show_version = true;
            config
        };

        let env_show_art = std::env::var("AXL_SHOW_ART").map_or(None, |val| Some(val == "true"));
        if env_show_art.is_some() {
            loaded_config.general.show_art = env_show_art;
        }
        info!("config: {:#?}", loaded_config);
        Ok(loaded_config)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_fs::{prelude::FileWriteStr, NamedTempFile};
    use rstest::{fixture, rstest};
    use similar_asserts::assert_eq;

    use crate::config::config_file::GeneralConfig;

    use super::AxlConfig;

    #[fixture]
    fn config_file_full() -> NamedTempFile {
        // Arrange
        let file = NamedTempFile::new("config_file_test_1.txt")
            .expect("test fixture tmp file can be created");
        file.write_str(
            "general:
    show_art: true
    show_version: false",
        )
        .expect("test fixture tmp file can be written to");
        file
    }

    #[fixture]
    fn config_file_empty() -> NamedTempFile {
        // Arrange
        let file = NamedTempFile::new("config_file_test_empty.txt")
            .expect("test fixture tmp file can be created");
        file.write_str("")
            .expect("test fixture tmp file can be written to");
        file
    }

    #[rstest]
    fn should_read_config_from_file(
        #[from(config_file_full)] config_file: NamedTempFile,
    ) -> Result<()> {
        let loaded_config = AxlConfig::from_file(config_file.path())?;

        assert_eq!(
            loaded_config,
            AxlConfig {
                general: GeneralConfig {
                    show_art: Some(true),
                    show_version: false,
                }
            }
        );

        Ok(())
    }

    #[rstest]
    fn should_default_empty_config_file(
        #[from(config_file_empty)] config_file: NamedTempFile,
    ) -> Result<()> {
        let loaded_config = AxlConfig::from_file(config_file.path())?;

        assert_eq!(
            loaded_config,
            AxlConfig {
                general: GeneralConfig {
                    show_art: None,
                    show_version: true
                }
            }
        );

        Ok(())
    }
}
