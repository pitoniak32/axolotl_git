use std::{
    fs::{self, File},
    path::PathBuf,
    process,
};

use anyhow::Result;
use axl_lib::{
    config::{AxlConfig, AxlContext},
    config_env::ConfigEnvKey,
    constants::{AxlColor, ASCII_ART},
    subcommand_project::ProjectSubcommand,
};
use bat::PrettyPrinter;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use rand::Rng;

const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
const PROJ_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS_PLATFORM: &str = std::env::consts::OS;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    args: SharedArgs,

    #[clap(skip)]
    context: AxlContext,
}

impl Cli {
    pub fn init() -> Result<Self> {
        let mut cli = Self::parse();
        env_logger::builder()
            .filter_level(cli.args.verbosity.log_level_filter())
            .parse_default_env()
            .init();
        log::debug!("cli_before_config_init: {cli:#?}");
        cli.set_config_path()?;
        let axl_config: AxlConfig = AxlConfig::from_file(&cli.context.config_path)?;
        if axl_config.general.show_art {
            Self::print_version_string(axl_config.general.show_art);
            Self::print_yaml_string(
                serde_yaml::to_string(&axl_config)
                    .expect("Should be able to convert struct to yaml string"),
            );
        }

        cli.context.config = axl_config;
        log::debug!("cli_after_config_init: {cli:#?}");

        Ok(cli)
    }

    fn print_version_string(show_art: bool) {
        println!(
            "{} {}{}{} {} {} {}\n{}\n",
            "~=".custom_color(AxlColor::HotPink.into()),
            PROJ_NAME.custom_color(AxlColor::TiffanyBlue.into()),
            "@".custom_color(AxlColor::HotPink.into()),
            PROJ_VERSION.custom_color(AxlColor::TiffanyBlue.into()),
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

    pub fn set_config_path(&mut self) -> Result<()> {
        if let Some(config_path) = &self.args.config_path {
            if let Ok(curr) = std::fs::canonicalize(config_path) {
                log::debug!("checking {}", curr.to_string_lossy());
                if !curr.exists() {
                    eprintln!(
                        "\n{}\n",
                        "Provided config path does not exist.".red().bold()
                    );
                    process::exit(1);
                }
                self.args.config_path = Some(curr.clone());
                self.context.config_path = curr;
            }
        } else {
            let mut path = PathBuf::try_from(ConfigEnvKey::XDGConfig)?;
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
            self.context.config_path = path.clone();
        }
        Ok(())
    }

    pub fn handle_command(self) -> Result<()> {
        if let Some(cmd) = self.command {
            Commands::handle(cmd, self.context, self.args)?;
        } else {
            eprintln!(
                "{}",
                "No command was provided! To see commands use `--help`."
                    .yellow()
                    .bold()
            );
            std::process::exit(1);
        }

        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(subcommand)]
    /// Commands for managing projects.
    ///
    /// All commands are using the selected project directory.
    Project(ProjectSubcommand),
}

impl Commands {
    fn handle(command: Self, context: AxlContext, args: SharedArgs) -> Result<()> {
        match command {
            Self::Project(subcommand) => {
                log::trace!("project...");
                ProjectSubcommand::handle_cmd(subcommand, context)?;
            }
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
struct SharedArgs {
    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    /// Override '$XDG_CONFIG_HOME/config.yml' or '$HOME/.axlrc.yml' defaults.
    #[arg(short, long)]
    config_path: Option<PathBuf>,
}
