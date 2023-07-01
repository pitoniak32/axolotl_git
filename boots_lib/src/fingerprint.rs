use clap::Args;

#[derive(Args, Debug)]
pub struct FingerprintOptions {
    /// Should force a project type
    #[arg(short, long)]
    pub force: Option<String>,
}
