use anyhow::{anyhow, Result};
use bat::{Input, PrettyPrinter};
use serde::{Deserialize, Serialize};
use std::fs;

use boots_lib::{config::BootsConfig, fingerprint::FingerprintOptions};
use clap::{Args, Command, Parser, Subcommand};
use colored::Colorize;

const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
const PROJ_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS_PLATFORM: &str = std::env::consts::OS;

const DASHES: &str = "--------------------------------";

fn main() -> Result<()> {
    // Somehow need to merge the cli arguments with the config file to allow for overriding values
    // with flags for testing.
    let cli = Cli::init()?;

    Cli::handle_command(cli.command)?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
#[deny(missing_docs)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    args: SharedArgs,

    #[clap(skip)]
    context: BootsContext,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct BootsContext {
    boots_config_path: String,
    boots_config: BootsConfig,
    git_commit_hash: String,
    build_id: String,
}

impl Cli {
    fn init() -> Result<Self> {
        Cli::print_version_string();
        let mut cli = Cli::parse();

        env_logger::builder()
            .filter_level(cli.args.verbosity.log_level_filter())
            .parse_default_env()
            .init();
        log::trace!("{cli:#?}");

        let boots_config: BootsConfig =
            serde_yaml::from_str(&fs::read_to_string(&cli.args.boots_config_path)?)?;
        log::trace!("{PROJ_NAME}_config: {:#?}", boots_config);
        // Cli::debug_file(
        //     "boots_config_file",
        //     serde_yaml::to_string(value)<BootsConfig>(&boots_config)?,
        // );

        let context = BootsContext {
            boots_config_path: cli.args.boots_config_path.clone(),
            boots_config: boots_config.clone(),
            git_commit_hash: "".to_string(),
            build_id: "".to_string(),
        };
        Cli::debug_file(&context.boots_config_path);
        cli.context = context;

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

    fn debug_file(file_path: &str) {
        // for t in PrettyPrinter::new().themes() {
        //     println!("theme: {t}");
        PrettyPrinter::new()
            .line_numbers(false)
            .grid(false)
            .header(false)
            .input_file(file_path)
            .theme("Nord")
            .print()
            .unwrap();
        // }

        println!();
        // log::debug!("\n{title}:\n{DASHES}\n{content}{DASHES}");
    }

    fn handle_command(command: Option<Commands>) -> Result<()> {
        if let Some(cmd) = command {
            Commands::handle(cmd)?;
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
    #[clap(subcommand)]
    Build(BuildCommands),
    /// Determine what type of project is being built.
    Fingerprint(FingerprintOptions),
}

impl Commands {
    fn handle(command: Commands) -> Result<()> {
        match command {
            Commands::Build(build_command) => {
                log::trace!("building...");
                match build_command {
                    BuildCommands::Test { metadata } => {
                        log::trace!("testing... {:#?}", metadata)
                    }
                    BuildCommands::Package { metadata } => {
                        log::trace!("packaging... {:#?}", metadata)
                    }
                    BuildCommands::Lint { metadata } => {
                        log::trace!("linting... {:#?}", metadata)
                    }
                }
            }
            Commands::Fingerprint(opts) => {
                println!("fingerprinting... {opts:#?}")
            }
        }
        Ok(())
    }
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
