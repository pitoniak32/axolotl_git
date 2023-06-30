use clap::{Parser, Subcommand, Args, Command};
use colored::Colorize;


fn main() {
    let cli = Cli::init();

    env_logger::builder()
        .filter_level(cli.args.verbosity.log_level_filter())
        .parse_default_env()
        .init();

    log::info!("{cli:#?}");
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

