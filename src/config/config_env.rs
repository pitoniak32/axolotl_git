use std::{env, path::PathBuf};

use thiserror::Error;
use tracing::trace;

use super::constants::{
    DEFAULT_MULTIPLEXER_KEY, HOME_DIR_KEY, XDG_CONFIG_HOME_DIR_KEY, XDG_DATA_HOME_DIR_KEY,
    XDG_STATE_HOME_DIR_KEY,
};

pub enum ConfigEnvKey {
    Home,
    XDGConfigHome,
    XDGDataHome,
    XDGStateHome,
    DefaultMultiplexerKey,
}

impl ConfigEnvKey {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Home => HOME_DIR_KEY,
            Self::XDGConfigHome => XDG_CONFIG_HOME_DIR_KEY,
            Self::XDGDataHome => XDG_DATA_HOME_DIR_KEY,
            Self::XDGStateHome => XDG_STATE_HOME_DIR_KEY,
            Self::DefaultMultiplexerKey => DEFAULT_MULTIPLEXER_KEY,
        }
    }

    pub const fn default_value(&self) -> &'static str {
        match self {
            Self::Home => "",
            Self::XDGConfigHome => "",
            Self::XDGDataHome => "",
            Self::XDGStateHome => "",
            Self::DefaultMultiplexerKey => "",
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
            ConfigEnvKey::XDGConfigHome => match env::var(ConfigEnvKey::XDGConfigHome.as_str()) {
                Ok(config_dir) => Ok(Self::from(config_dir)),
                Err(_err) => {
                    let mut home = Self::try_from(ConfigEnvKey::Home)?;
                    home.push(".config");
                    trace!(
                        "Error: error reading ${}. Using [{}]",
                        ConfigEnvKey::XDGConfigHome.as_str(),
                        home.as_os_str().to_string_lossy()
                    );
                    Ok(home)
                }
            },
            ConfigEnvKey::XDGDataHome => Ok(Self::from(
                env::var(ConfigEnvKey::XDGDataHome.as_str())
                    .expect("XDG_DATA_HOME env var should be set"),
            )),
            ConfigEnvKey::XDGStateHome => Ok(Self::from(
                env::var(ConfigEnvKey::XDGStateHome.as_str())
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
