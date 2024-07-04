use tracing_subscriber::EnvFilter;

pub fn get_global_filter() -> EnvFilter {
    if std::env::var("RUST_LOG").is_ok() {
        // Read global subscriber filter from `RUST_LOG`
        EnvFilter::builder().from_env_lossy()
    } else {
        "warn,axolotl_git=info"
            .parse()
            .expect("valid EnvFilter value can be parsed")
    }
}
