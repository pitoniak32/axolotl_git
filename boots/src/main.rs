use std::fs;
use anyhow::Result;

use boots_lib::config::BootsConfig;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
const PROJ_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS_PLATFORM: &str = std::env::consts::OS;

fn main() -> Result<()> {
    let cli = Cli::init();

    env_logger::builder()
        .filter_level(cli.args.verbosity.log_level_filter())
        .parse_default_env()
        .init();
    log::trace!("{cli:#?}");

    // Somehow need to merge the cli arguments with the config file to allow for overriding values
    // with flags for testing.

    let boots_config: BootsConfig = serde_yaml::from_str(&fs::read_to_string(&cli.args.boots_config_path).unwrap()).unwrap();
    log::trace!("{PROJ_NAME}_config: {:#?}", boots_config);

    log::debug!("{}", serde_yaml::to_string::<BootsConfig>(&boots_config).unwrap());

    Cli::handle_command(cli.command.unwrap())?;

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
    fn init() -> Self {
        Cli::print_version_string();
        Cli::parse()
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

    fn handle_command(command: Commands) -> Result<()> {
        match command {
            Commands::Build(build_command) => {
                log::trace!("building...");
                match build_command {
                    BuildCommands::Test { metadata } => {
                        log::trace!("testing... {:#?}", metadata)
                    },
                    BuildCommands::Package { metadata } => {
                        log::trace!("packaging... {:#?}", metadata)
                    },
                    BuildCommands::Lint { metadata } => {
                        log::trace!("linting... {:#?}", metadata)
                    },
                }
            },
            Commands::Fingerprint => {
                println!("fingerprinting...")
            }, 
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
