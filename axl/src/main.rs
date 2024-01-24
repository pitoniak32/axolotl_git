use anyhow::Result;
use cli::Cli;

pub mod cli;

fn main() -> Result<()> {
    // Somehow need to merge the cli arguments with the config file to allow for overriding values
    // with flags for testing.
    let cli = Cli::init()?;
    cli.handle_command()?;

    Ok(())
}
