use std::{env, path::PathBuf};

use thiserror::Error;

pub enum ConfigEnvKey {
    Home,
    XDGConfig,
    XDGData,
    XDGState,
}

impl ConfigEnvKey {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Home => "HOME",
            Self::XDGConfig => "XDG_CONFIG_HOME",
            Self::XDGData => "XDG_DATA_HOME",
            Self::XDGState => "XDG_STATE_HOME",
        }
    }

    pub const fn default_value(&self) -> &'static str {
        match self {
            Self::Home => "",
            Self::XDGConfig => "",
            Self::XDGData => "",
            Self::XDGState => "",
        }
    }
}

const DEFAULT_PANIC_MSG: &str =
    "Check the impl block for the type you are trying to use and make sure the key is implemented.";

impl TryFrom<ConfigEnvKey> for PathBuf {
    type Error = ConfigError;
    fn try_from(env_key: ConfigEnvKey) -> Result<Self, ConfigError> {
        match env_key {
            ConfigEnvKey::Home => Ok(Self::from(
                env::var(ConfigEnvKey::Home.as_str()).expect("HOME env var should be set"),
            )),
            ConfigEnvKey::XDGConfig => match env::var(ConfigEnvKey::XDGConfig.as_str()) {
                Ok(config_dir) => Ok(Self::from(config_dir)),
                Err(_err) => {
                    let mut home = Self::try_from(ConfigEnvKey::Home)?;
                    home.push(".config");
                    log::trace!(
                        "Error: error reading ${}. Using [{}]",
                        ConfigEnvKey::XDGConfig.as_str(),
                        home.as_os_str().to_string_lossy()
                    );
                    Ok(home)
                }
            },
            ConfigEnvKey::XDGData => Ok(Self::from(
                env::var(ConfigEnvKey::XDGData.as_str())
                    .expect("XDG_DATA_HOME env var should be set"),
            )),
            ConfigEnvKey::XDGState => Ok(Self::from(
                env::var(ConfigEnvKey::XDGState.as_str())
                    .expect("XDG_STATE_HOME env var should be set"),
            )),
            #[allow(unreachable_patterns)]
            // This is allowed because not all enum variants are guaranteed to be this type in the
            // futrue.
            _ => panic!("this key cannot be converted to PathBuf. {DEFAULT_PANIC_MSG}"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("genertic")]
    Generic,
}
