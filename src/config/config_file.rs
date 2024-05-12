use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tracing::{debug, instrument};

use crate::config::config_env::ConfigEnvKey;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AxlContext {
    pub config_path: PathBuf,
    pub config: AxlConfig,
}

/// Command Line Flags Should Overtake File Values.
/// How can I show that a config option is available
/// in the config file and in the cli flags?
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct AxlConfig {
    pub general: GeneralConfig,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct GeneralConfig {
    pub decoration: DecorationOption,
}

#[derive(Serialize, Deserialize, Default, ValueEnum, Debug, Clone, PartialEq, Eq, Display)]
#[serde(rename_all = "kebab-case")]
pub enum DecorationOption {
    None,
    VersionBanner,
    Art,
    #[default]
    All,
}

#[derive(Serialize, Deserialize, Default, ValueEnum, Debug, Clone, PartialEq, Eq, Display)]
#[serde(rename_all = "kebab-case")]
pub enum OnError {
    /// Just print error and continue.
    #[default]
    None,
    /// Prompt user to press key to continue.
    Pause,
    /// 500 millisecond delay.
    ShortDelay,
    /// 5000 millisecond delay.
    LongDelay,
}

impl AxlConfig {
    #[instrument(err)]
    pub fn from_file(config_path: &Path) -> Result<Self> {
        let config_string = &fs::read_to_string(config_path)?;
        let mut loaded_config = if !config_string.trim().is_empty() {
            serde_yaml::from_str(config_string)?
        } else {
            Self::default()
        };

        if let Ok(decoration_env) = DecorationOption::try_from(ConfigEnvKey::Decorations) {
            loaded_config.general.decoration = decoration_env;
        }

        debug!("config: {:#?}", loaded_config);
        Ok(loaded_config)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_fs::{prelude::FileWriteStr, NamedTempFile};
    use rstest::{fixture, rstest};
    use similar_asserts::assert_eq;

    use crate::config::config_file::{DecorationOption, GeneralConfig};

    use super::AxlConfig;

    #[fixture]
    fn config_file_full() -> NamedTempFile {
        // Arrange
        let file = NamedTempFile::new("config_file_test_1.txt")
            .expect("test fixture tmp file can be created");
        file.write_str(
            "general:
    decoration: version-banner",
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
                    decoration: DecorationOption::VersionBanner
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
                    decoration: DecorationOption::All
                }
            }
        );

        Ok(())
    }
}
