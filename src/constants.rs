use colored::CustomColor;
use serde_derive::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub const fn version_str() -> &'static str {
    if option_env!("GIT_SHA_SHORT").is_some() && !env!("GIT_SHA_SHORT").is_empty() {
        concat!(env!("CARGO_PKG_VERSION"), "-dev-", env!("GIT_SHA_SHORT"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

pub const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
pub const OS_PLATFORM: &str = std::env::consts::OS;
pub const VERSION_STR: &str = version_str();
pub const GIT_SHA_LONG: &str = env!("GIT_SHA_LONG");
pub const GIT_SHA_SHORT: &str = env!("GIT_SHA_SHORT");

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
