pub mod bench;
pub mod eval;

use std::path::PathBuf;

pub use bench::BenchArgs;
use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
pub use eval::EvalArgs;

#[derive(Parser, Debug)]
pub struct SharedArgs {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Path to bench results
    #[arg(short, long, default_value = "results.ndjson")]
    pub results: PathBuf,

    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}
