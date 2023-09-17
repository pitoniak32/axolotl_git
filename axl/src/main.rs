use anyhow::Result;
use bat::PrettyPrinter;
use serde::{Deserialize, Serialize};
use rand::Rng;

use axl_lib::{config::AxlConfig, constants::ASCII_ART};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

const PROJ_NAME: &str = env!("CARGO_PKG_NAME");
const PROJ_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS_PLATFORM: &str = std::env::consts::OS;

fn main() -> Result<()> {
    // Somehow need to merge the cli arguments with the config file to allow for overriding values
    // with flags for testing.
    let cli = Cli::init()?;

    cli.handle_command()?;

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
    context: AxlContext,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct AxlContext {
    config_path: String,
    config: AxlConfig,
}

impl Cli {
    fn init() -> Result<Self> {
        let mut cli = Cli::parse();
        env_logger::builder()
            .filter_level(cli.args.verbosity.log_level_filter())
            .parse_default_env()
            .init();
        log::debug!("cli_before_config_init: {cli:#?}");
        let axl_config: AxlConfig = AxlConfig::new(&cli.args.config_path)?;
        Cli::print_version_string(axl_config.show_art);
        Cli::print_yaml_string(
            serde_yaml::to_string(&axl_config)
                .expect("Should be able to convert struct to yaml string"),
        );

        let context = AxlContext {
            config_path: cli.args.config_path.clone(),
            config: axl_config.clone(),
        };
        cli.context = context;
        log::debug!("cli_after_config_init: {cli:#?}");

        Ok(cli)
    }

    fn print_version_string(show_art: bool) {
        println!(
            "{}{}{} {} {}\n{}\n",
            PROJ_NAME.blue(),
            "@".green(),
            PROJ_VERSION.blue(),
            "on".green(),
            OS_PLATFORM.blue(),
            if show_art { ASCII_ART[rand::thread_rng().gen_range(0..ASCII_ART.len())] } else {""},
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
            .unwrap();
        println!();
    }

    fn handle_command(self) -> Result<()> {
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
}

impl Commands {
    fn handle(command: Commands, _context: AxlContext) -> Result<()> {
        match command {
            Commands::Build => {
                log::trace!("building...");
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
