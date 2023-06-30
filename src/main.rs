use clap::{Parser, Subcommand, Args, Command};
use colored::Colorize;
use serde_derive::{Serialize, Deserialize};


fn main() {
    let cli = Cli::init();

    env_logger::builder()
        .filter_level(cli.args.verbosity.log_level_filter())
        .parse_default_env()
        .init();

    log::info!("{cli:#?}");
}

#[derive(Serialize, Deserialize, Debug)]
struct BootsConfig {
    version: String,
    project_name: String,
    project_info: ProjectTypes,

}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "project_type", content = "project_config")]
enum ProjectTypes {
    NPM {
        allowed_targets: Vec<ArtifactTargets>,
    },
    YARN {
        allowed_targets: Vec<ArtifactTargets>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
enum ArtifactTargets {
    Image,
    Tarball,
}


#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
   
    #[clap(flatten)]
    args: SharedArgs
}

impl Cli {
    fn init() -> Self {
        Cli::print_version_string();
        Cli::parse()
    }

    fn print_version_string() {
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let platform = std::env::consts::OS;
        println!("{}{}{} {} {}\n", name.blue(), "@".green(), version.blue(), "on".green(), platform.blue());
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(subcommand)]
    Build(BuildCommands)
}

#[derive(Subcommand, Debug)]
enum BuildCommands {
    Test {
        #[clap(short, long)]
        metadata: String
    },
    Package {
        #[clap(short, long)]
        metadata: String
    },
    Lint {
        #[clap(short, long)]
        metadata: String
    },
}

#[derive(Args, Debug)]
struct SharedArgs {

    #[clap(flatten)] 
    verbosity: clap_verbosity_flag::Verbosity,
}

