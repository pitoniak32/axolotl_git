use anyhow::Result;
use axl_lib::{
    config::AxlConfig,
    constants::{AxlColor, ASCII_ART},
    project::ProjectSubcommand,
};
use bat::PrettyPrinter;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct AxlContext {
    config_path: String,
    config: AxlConfig,
}

impl Cli {
    pub fn init() -> Result<Self> {
        let mut cli = Self::parse();
        env_logger::builder()
            .filter_level(cli.args.verbosity.log_level_filter())
            .parse_default_env()
            .init();
        log::debug!("cli_before_config_init: {cli:#?}");
        let axl_config: AxlConfig = AxlConfig::new(&cli.args.config_path)?;
        Self::print_version_string(axl_config.show_art);
        Self::print_yaml_string(
            serde_yaml::to_string(&axl_config)
                .expect("Should be able to convert struct to yaml string"),
        );

        let context = AxlContext {
            config_path: cli.args.config_path.clone(),
            config: axl_config,
        };
        cli.context = context;
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

    pub fn handle_command(self) -> Result<()> {
        if let Some(cmd) = self.command {
            Commands::handle(cmd, self.context)?;
        } else {
            println!(
                "{}",
                "No command was provided! To see commands use `--help`."
                    .yellow()
                    .bold()
            );
        }
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Commands related to building projects
    Build,

    #[clap(subcommand)]
    /// Commands for managing projects.
    Project(ProjectSubcommand),
}

impl Commands {
    fn handle(command: Self, _context: AxlContext) -> Result<()> {
        match command {
            Self::Build => {
                log::trace!("building...");
            }
            Self::Project(_subcommand) => {
                // ProjectSubcommand::handle_cmd(_subcommand, )?;
                log::trace!("project...");
            }
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
struct SharedArgs {
    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[arg(short, long, default_value = "./cfg_example.yml")]
    config_path: String,

    #[arg(short, long, default_value = ".")]
    project_root: String,
}
