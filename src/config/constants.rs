use colored::{Colorize, CustomColor};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const HOME_DIR_KEY: &str = "HOME";
pub const XDG_CONFIG_HOME_DIR_KEY: &str = "XDG_CONFIG_HOME";
pub const XDG_DATA_HOME_DIR_KEY: &str = "XDG_DATA_HOME";
pub const XDG_STATE_HOME_DIR_KEY: &str = "XDG_STATE_HOME";
pub const DEFAULT_MULTIPLEXER_KEY: &str = "AXL_DEFAULT_MULTIPLEXER";
pub const DEFAULT_PROJECTS_CONFIG_PATH_KEY: &str = "AXL_PROJECTS_CONFIG_PATH";

// version string constants
pub const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
pub const OS_PLATFORM: &str = std::env::consts::OS;
pub const AXL_VERSION_STR: &str = version_str();
pub const AXL_GIT_SHA_LONG: &str = env!("GIT_SHA_LONG");
pub const AXL_GIT_SHA_SHORT: &str = env!("GIT_SHA_SHORT");

pub const fn version_str() -> &'static str {
    if option_env!("GIT_SHA_SHORT").is_some() && !env!("GIT_SHA_SHORT").is_empty() {
        concat!(env!("CARGO_PKG_VERSION"), "-dev-", env!("GIT_SHA_SHORT"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

pub fn print_version_string(show_art: bool) {
    eprintln!(
        "{} {}{}{} {} {} {}\n{}",
        "~=".custom_color(AxlColor::HotPink.into()),
        PROJ_NAME.custom_color(AxlColor::TiffanyBlue.into()),
        "@".custom_color(AxlColor::HotPink.into()),
        AXL_VERSION_STR.custom_color(AxlColor::TiffanyBlue.into()),
        "on".custom_color(AxlColor::HotPink.into()),
        OS_PLATFORM.custom_color(AxlColor::TiffanyBlue.into()),
        "=~".custom_color(AxlColor::HotPink.into()),
        if show_art {
            let mut colors = AxlColor::iter();
            let rand_color_index = rand::thread_rng().gen_range(0..colors.len());
            let rand_art_index = rand::thread_rng().gen_range(0..ASCII_ART.len());
            ASCII_ART[rand_art_index].to_string().custom_color(
                colors
                    .nth(rand_color_index)
                    .unwrap_or(AxlColor::TiffanyBlue)
                    .into(),
            )
        } else {
            "".normal()
        },
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliInfo<'a> {
    pub version: &'a str,
    pub commit: &'a str,
    pub os_platform: &'a str,
}

#[derive(EnumIter)]
pub enum AxlColor {
    HotPink,
    TiffanyBlue,
    Mint,
    Yellow,
}

impl From<AxlColor> for CustomColor {
    fn from(axl_color: AxlColor) -> Self {
        match axl_color {
            AxlColor::HotPink => Self {
                r: 255,
                g: 174,
                b: 188,
            },
            AxlColor::TiffanyBlue => Self {
                r: 160,
                g: 231,
                b: 229,
            },
            AxlColor::Mint => Self {
                r: 180,
                g: 248,
                b: 200,
            },
            AxlColor::Yellow => Self {
                r: 251,
                g: 231,
                b: 198,
            },
        }
    }
}

pub const ASCII_ART: [&str; 2] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡏⠈⢱⠀⠀⡖⠲⣀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠋⠹⡇⠀⡸⢠⠞⠳⠆⠈⡆⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⠤⠤⠤⠤⠤⢬⣇⢀⣿⣚⢳⡏⠀⢰⠃⡴⠛⢦⠀⠀⠀⠀⠀⠀
⠀⡠⣄⢠⠒⣄⠐⢄⠀⠀⣠⠴⠋⠁⠀⠀⠀⠀⠀⠀⠀⠈⠻⣿⣸⡟⢣⣠⣿⣯⣤⡔⠃⠀⠀⠀⠀⠀⠀
⠘⣇⠈⢻⡀⠸⡄⠈⣆⠞⠁⠀⠀⠀⠀⠀⠀⠀⣶⣶⣄⡀⠀⠙⠿⣿⣿⣻⡿⠋⢹⠟⠉⡗⠂⠀⠀⠀⠀
⢴⠚⠢⢤⣿⣧⣽⣶⣏⡀⠀⠀⠀⠀⠀⠀⣀⠀⠘⠿⡭⢯⠆⠐⢲⣿⣾⣿⢁⣶⣏⡠⠞⢳⠉⢩⠏⠀⠀
⠈⡗⠒⣿⡈⣿⡍⣿⣿⣷⠀⣀⣴⣻⣶⠋⠉⠀⠀⠀⠀⠀⠀⠀⠀⢠⡾⠻⠿⣍⠉⣴⠒⠋⢀⠇⠀⠆⠀
⢠⠽⠦⠈⣳⣌⣷⣿⠷⠟⠀⠀⠀⠀⠀⠀⠀⠀⣠⢶⡶⢤⣀⠀⢀⡼⠙⣶⣤⠟⠓⠋⠀⠀⠸⡀⠀⢦⠀
⠘⠂⣤⡔⠛⢯⣙⣿⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢧⡃⠀⠈⠙⠛⠓⠒⠛⠦⣀⠀⠀⠀⠀⠀⣇⠀⠘⡀
⠀⠸⢅⣙⠶⢲⠟⠻⢿⡷⣄⣀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠂⠀⠀⠀⠀⠀⠀⠀⠈⠉⠉⣳⠀⢀⡏⠀⢠⠇
⠀⠀⠀⠈⠀⠸⠤⠚⠛⠁⢾⠋⠉⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣴⢛⣉⠴⠛⠀⢀⡞⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠒⠒⠦⠴⠦⠶⢤⣀⠀⠀⠀⠀⠀⠀⠀⢠⠿⣍⡉⠁⠀⠀⣀⡤⠊⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠲⠦⣄⣀⣀⡤⠴⠒⠚⠋⠉⠉⠉⠁⠀⠀⠀⠀",
    "⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⣀⣀⡤⠤⠤⠤⣄⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⢴⠋⠙⠳⣤⡀⣠⠖⠋⠁⠀⠀⠀⠀⠀⠀⠀⠉⠓⠤⡀⣠⡴⠟⠛⣷⠀⠀
⠀⠀⠈⠳⢤⣀⢈⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢏⣀⣠⡴⠋⠀⠀
⢀⣠⣤⣄⣄⣉⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣇⣡⣤⡴⣦⣀
⢹⣅⡀⠀⠀⠈⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡏⠁⠀⢀⣠⠏
⠀⠈⠙⠉⢋⣉⣇⠀⠀⣾⣷⠄⠀⠀⠀⠀⠀⠀⠀⢴⣿⡆⠀⢀⣿⡙⠋⠋⠀⠀
⠀⠀⢤⠶⠋⠉⢈⣦⡀⠈⠉⠀⠀⠀⠉⠉⠉⠀⠀⠀⠉⠀⣠⣎⠈⠉⠛⢷⡀⠀
⠀⠀⠻⣤⣤⠶⠋⠀⠈⠑⠠⠄⣀⣀⣀⣀⣀⣀⡀⠤⠐⠉⠀⠈⠻⠶⠖⠶⠃⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠃⠤⡀⠀⡠⠤⢱⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡎⠀⠉⠁⠀⠉⠉⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⢀⠤⡀⠀⡠⢄⢀⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠢⣀⣀⣀⣈⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡌⡇⠀⣎⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢡⡇⣸⠜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠓⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
];
