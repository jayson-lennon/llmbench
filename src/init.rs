pub mod error_reporting;

pub use error_reporting::init_error_stack;

use clap_verbosity_flag::{Verbosity, WarnLevel};
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(verbosity: Verbosity<WarnLevel>) {
    use std::env;
    use tracing_subscriber::EnvFilter;

    let filter = match env::var("RUST_LOG") {
        Ok(filter_str) => filter_str,
        Err(_) => format!("llmbench={verbosity}"),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(filter)))
        .init();
}
