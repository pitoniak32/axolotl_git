use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::trace;

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

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneralConfig {
    #[serde(default = "art_default")]
    pub show_art: bool,
}

const fn art_default() -> bool {
    // Set to false since most commands pull up a prompt immediately
    false
}

impl AxlConfig {
    pub fn from_file(config_path: &Path) -> Result<Self> {
        trace!("loading config from {}...", config_path.to_string_lossy());
        let config_string = &fs::read_to_string(config_path)?;
        let mut loaded_config = if !config_string.trim().is_empty() {
            serde_yaml::from_str(config_string)?
        } else {
            Self::default()
        };
        loaded_config.general.show_art = std::env::var("AXL_SHOW_ART")
            .map_or(loaded_config.general.show_art, |val| val == "true");
        trace!("config: {:#?}", loaded_config);
        trace!("config loaded!");
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
    show_art: true",
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
                general: GeneralConfig { show_art: true }
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
                general: GeneralConfig { show_art: false }
            }
        );

        Ok(())
    }
}
