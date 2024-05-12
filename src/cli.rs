use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::Result;
use axl_lib::{
    config::{
        config_env::ConfigEnvKey,
        config_file::{AxlConfig, AxlContext, DecorationOption, OnError},
        constants::{
            print_art, print_version_string, CliInfo, AXL_GIT_SHA_LONG, AXL_VERSION_STR,
            OS_PLATFORM,
        },
    },
    error::Error,
    helper::formatted_print,
    multiplexer::Multiplexer,
    project::subcommand::{OutputFormat, ProjectSubcommand, SessionArgs},
};
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::LogLevel;
use colored::Colorize;
use strum_macros::Display;
use tracing::{debug, instrument};

#[derive(Parser, Debug)]
#[command(author, version = AXL_VERSION_STR, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(flatten)]
    pub args: SharedArgs,

    #[clap(skip)]
    ctx: AxlContext,
}

impl Cli {
    #[instrument(skip_all, err)]
    pub fn init(mut self) -> Result<Self> {
        debug!("cli_before_config_init: {self:#?}");
        self.set_config_path()?;
        let axl_config: AxlConfig = AxlConfig::from_file(&self.ctx.config_path)?;
        self.ctx.config = axl_config;
        self.ctx.config.general.decoration = self.args.decoration.clone();
        debug!("cli_after_config_init: {self:#?}");

        match self.ctx.config.general.decoration {
            DecorationOption::None => (),
            DecorationOption::VersionBanner => {
                print_version_string();
            }
            DecorationOption::Art => {
                print_art();
            }
            DecorationOption::All => {
                print_version_string();
                print_art();
            }
        };

        Ok(self)
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
                    Err(Error::ConfigPathDoesNotExist)?
                }
                self.args.config_path = Some(curr.clone());
                self.ctx.config_path = curr;
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
            self.ctx.config_path.clone_from(&path);
        }
        Ok(())
    }

    #[instrument(skip(self), fields(command.name = %self.command), err)]
    pub fn handle_command(&self) -> Result<()> {
        Commands::handle(&self.command, &self.ctx, &self.args)?;
        Ok(())
    }
}

#[derive(Subcommand, Debug, Display)]
pub enum Commands {
    #[clap(subcommand, visible_alias = "p")]
    /// Commands for managing projects.
    ///
    /// All commands are using the selected project directory.
    #[strum()]
    Project(ProjectSubcommand),

    #[clap(visible_alias = "ls")]
    /// Command to list sessions
    ///
    /// list your current multiplexer sessions.
    #[strum()]
    List {
        #[clap(flatten)]
        sess_args: SessionArgs,
    },

    Info {
        #[arg(short, long, value_enum, default_value_t=OutputFormat::Json)]
        output: OutputFormat,
    },
}

impl Commands {
    #[instrument(skip(command, context, _args), err)]
    fn handle(command: &Self, context: &AxlContext, _args: &SharedArgs) -> Result<()> {
        match command {
            Self::Project(subcommand) => {
                ProjectSubcommand::handle_cmd(subcommand, context)?;
            }
            Self::List { sess_args } => {
                println!("{}", sess_args.multiplexer.get_sessions()?.join("\n"));
            }
            Self::Info { output } => {
                let info = CliInfo {
                    version: AXL_VERSION_STR,
                    os_platform: OS_PLATFORM,
                    commit: AXL_GIT_SHA_LONG,
                };
                formatted_print(output, info)?;
            }
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct SharedArgs {
    #[clap(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<OffLevel>,

    /// Override '$XDG_CONFIG_HOME/axl/config.yml' or '$HOME/.axlrc.yml' defaults.
    #[arg(short, long, env("AXL_CONFIG_PATH"))]
    config_path: Option<PathBuf>,

    /// Helpful for tmux popup prompts to see why a command failed.
    #[arg(long, value_enum, default_value_t)]
    pub on_error: OnError,

    /// Control which decorations are displayed.
    #[arg(long, value_enum, default_value_t)]
    pub decoration: DecorationOption,
}

#[derive(Debug)]
pub struct OffLevel;

impl LogLevel for OffLevel {
    fn default() -> Option<tracing_log::log::Level> {
        None
    }
}
