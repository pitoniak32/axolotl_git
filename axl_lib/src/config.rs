use anyhow::Result;
use std::fs;

use serde::{Deserialize, Serialize};

fn art_default() -> bool {
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
        let mut axl_config: AxlConfig = serde_yaml::from_str(&fs::read_to_string(path)?)?;
        log::debug!("axl_config_before: {:#?}", &axl_config);
        let axl_inline = AxlConfig {
            show_art: std::env::var("AXL_SHOW_ART")
                .map_or(axl_config.show_art, |val| val == "true"),
        };
        axl_config = AxlConfig::merge(Some(axl_inline), Some(axl_config)).unwrap();
        log::debug!("axl_config_after: {:#?}", &axl_config);
        Ok(axl_config)
    }

    pub fn merge(
        axl_config_inline: Option<AxlConfig>,
        axl_config_file: Option<AxlConfig>,
    ) -> Option<Self> {
        match (axl_config_inline, axl_config_file) {
            (None, None) => None,
            (None, Some(axl_file)) => Some(axl_file),
            (Some(axl_inline), None) => Some(axl_inline),
            (Some(_axl_inline), Some(_axl_file)) => {
                log::debug!("merging configs...");
                // Read the config file
                let merged = AxlConfig {
                    show_art: _axl_inline.show_art,
                };
                Some(merged)
            }
        }
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
