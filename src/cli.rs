use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::Result;
use axl_lib::{
    config::{
        config_env::ConfigEnvKey,
        config_file::{AxlConfig, AxlContext},
    },
    constants::{AxlColor, ASCII_ART},
    error::AxlError,
    project::subcommand::ProjectSubcommand,
};
use bat::PrettyPrinter;
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::LogLevel;
use colored::Colorize;
use rand::Rng;
use strum_macros::Display;
use tracing::{debug, info, instrument};

const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
const OS_PLATFORM: &str = std::env::consts::OS;
const VERSION_STR: &str = concat!(env!("CARGO_PKG_VERSION"), "-", env!("GIT_HASH"));

#[derive(Parser, Debug)]
#[command(author, version = VERSION_STR, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(flatten)]
    pub args: SharedArgs,

    #[clap(skip)]
    context: AxlContext,
}

impl Cli {
    #[instrument(skip_all, err)]
    pub fn init(mut self) -> Result<Self> {
        info!("cli_before_config_init: {self:#?}");
        let _ = &self.set_config_path()?;
        let show_art_arg = &self.context.config.general.show_art;
        let axl_config: AxlConfig = AxlConfig::from_file(&self.context.config_path)?;
        let show_art = *show_art_arg && axl_config.general.show_art;
        Self::print_version_string(show_art);
        if show_art {
            Self::print_yaml_string(
                serde_yaml::to_string(&axl_config)
                    .expect("Should be able to convert struct to yaml string"),
            );
        }

        self.context.config = axl_config;
        debug!("cli_after_config_init: {self:#?}");

        Ok(self)
    }

    #[instrument]
    fn print_version_string(show_art: bool) {
        eprintln!(
            "{} {}{}{} {} {} {}\n{}",
            "~=".custom_color(AxlColor::HotPink.into()),
            PROJ_NAME.custom_color(AxlColor::TiffanyBlue.into()),
            "@".custom_color(AxlColor::HotPink.into()),
            VERSION_STR.custom_color(AxlColor::TiffanyBlue.into()),
            "on".custom_color(AxlColor::HotPink.into()),
            OS_PLATFORM.custom_color(AxlColor::Mint.into()),
            "=~".custom_color(AxlColor::HotPink.into()),
            if show_art {
                ASCII_ART[rand::thread_rng().gen_range(0..ASCII_ART.len())]
                    .custom_color(AxlColor::TiffanyBlue.into())
            } else {
                "".normal()
            },
        );
    }

    #[instrument(skip_all)]
    fn print_yaml_string(content: String) {
        let bytes = content.as_bytes();
        PrettyPrinter::new()
            .language("yaml")
            .line_numbers(false)
            .grid(false)
            .header(false)
            .theme("Nord")
            .input_from_bytes(bytes)
            .print()
            .expect("yaml pretty printer should not fail");
        println!();
    }

    #[instrument(skip_all, err)]
    pub fn set_config_path(&mut self) -> Result<()> {
        if let Some(config_path) = &self.args.config_path {
            if let Ok(curr) = std::fs::canonicalize(config_path) {
                debug!("checking {}", curr.to_string_lossy());
                if !curr.exists() {
                    eprintln!(
                        "\n{}\n",
                        "Provided config path does not exist.".red().bold()
                    );
                    Err(AxlError::ConfigPathDoesNotExist)?
                }
                self.args.config_path = Some(curr.clone());
                self.context.config_path = curr;
            }
        } else {
            let mut path = PathBuf::try_from(ConfigEnvKey::XDGConfigHome)?;
            if path.exists() {
                path.push("axl");
                if !path.exists() {
                    fs::create_dir(&path)?;
                }
                path.push("config.yml");
                if !path.exists() {
                    File::create(&path)?;
                }
            } else {
                let mut path = PathBuf::try_from(ConfigEnvKey::Home)?;
                if path.exists() {
                    path.push(".axlrc.yml");
                    if !path.exists() {
                        File::create(&path)?;
                    }
                }
            }
            self.args.config_path = Some(path.clone());
            self.context.config_path.clone_from(&path);
        }
        Ok(())
    }

    #[instrument(skip(self), fields(command.name = %self.command), err)]
    pub fn handle_command(self) -> Result<()> {
        Commands::handle(self.command, self.context, self.args)?;
        Ok(())
    }
}

#[derive(Subcommand, Debug, Display)]
pub enum Commands {
    #[clap(subcommand)]
    /// Commands for managing projects.
    ///
    /// All commands are using the selected project directory.
    #[strum()]
    Project(ProjectSubcommand),
}

impl Commands {
    #[instrument(skip(command, context, _args), err)]
    fn handle(command: Self, context: AxlContext, _args: SharedArgs) -> Result<()> {
        match command {
            Self::Project(subcommand) => {
                ProjectSubcommand::handle_cmd(subcommand, context)?;
            }
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct SharedArgs {
    #[clap(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<OffLevel>,

    /// Override '$XDG_CONFIG_HOME/config.yml' or '$HOME/.axlrc.yml' defaults.
    #[arg(short, long)]
    config_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct OffLevel;

impl LogLevel for OffLevel {
    fn default() -> Option<tracing_log::log::Level> {
        None
    }
}
