use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    use std::env;
    use tracing_subscriber::EnvFilter;

    let filter = match env::var("RUST_LOG") {
        Ok(filter_str) => filter_str,
        Err(_) => "llmbench=info".to_string(),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(filter)))
        .init();
}
