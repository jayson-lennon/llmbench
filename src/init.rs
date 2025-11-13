pub fn init_tracing() {
    use std::env;
    use tracing_subscriber::EnvFilter;

    let filter = match env::var("RUST_LOG") {
        Ok(filter_str) => filter_str,
        Err(_) => "llmbench=trace,bench=trace".to_string(),
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}
