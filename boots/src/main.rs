use anyhow::{Result, anyhow};
use std::fs;

use boots_lib::config::BootsConfig;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
const PROJ_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS_PLATFORM: &str = std::env::consts::OS;

const DASHES: &str = "--------------------------------";

fn main() -> Result<()> {
    // Somehow need to merge the cli arguments with the config file to allow for overriding values
    // with flags for testing.
    match Cli::init() {
        Ok(cli) => match cli.handle_command() {
            Ok(_) => {
                log::trace!("Successful!");
            },
            Err(e) => {
                log::error!(
                    "An error occured while handing command: {e:#?}"
                );
                std::process::exit(1);
            }
        },
        Err(e) => {
            log::error!("An error occured during cli initalization: {e:#?}");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    args: SharedArgs,
}

impl Cli {
    fn init() -> Result<Self> {
        Cli::print_version_string();
        let cli = Cli::parse();

        env_logger::builder()
            .filter_level(cli.args.verbosity.log_level_filter())
            .parse_default_env()
            .init();
        log::trace!("{cli:#?}");

        let boots_config: BootsConfig =
            serde_yaml::from_str(&fs::read_to_string(&cli.args.boots_config_path)?)?;
        log::trace!("{PROJ_NAME}_config: {:#?}", boots_config);
        Cli::debug_file(
            "boots_config_file",
            serde_yaml::to_string::<BootsConfig>(&boots_config)?,
        );

        Ok(cli)
    }

    fn print_version_string() {
        println!(
            "{}{}{} {} {}\n",
            PROJ_NAME.blue(),
            "@".green(),
            PROJ_VERSION.blue(),
            "on".green(),
            OS_PLATFORM.blue()
        );
    }

    fn debug_file(title: &str, content: String) {
        log::debug!("\n{title}:\n{DASHES}\n{content}{DASHES}");
    }

    fn handle_command(&self) -> Result<()> {
        let command = self.command.as_ref().unwrap();
        match command {
            Commands::Build(build_command) => {
                log::trace!("building...");
                match build_command {
                    BuildCommands::Test { metadata } => {
                        log::trace!("testing... {:#?}", metadata);
                        return Err(anyhow!("test failure"));
                    },
                    BuildCommands::Package { metadata } => {
                        log::trace!("packaging... {:#?}", metadata)
                    },
                    BuildCommands::Lint { metadata } => {
                        log::trace!("linting... {:#?}", metadata)
                    },
                }
            }
            Commands::Fingerprint => {
                println!("fingerprinting...")
            }
        }
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(subcommand)]
    Build(BuildCommands),

    Fingerprint,
}

#[derive(Subcommand, Debug)]
enum BuildCommands {
    Test {
        #[clap(short, long)]
        metadata: String,
    },
    Package {
        #[clap(short, long)]
        metadata: String,
    },
    Lint {
        #[clap(short, long)]
        metadata: String,
    },
}

#[derive(Args, Debug)]
struct SharedArgs {
    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[arg(short, long, default_value = "./boots_cfg.yml")]
    boots_config_path: String,

    #[arg(short, long, default_value = ".")]
    project_root: String,
}
