use std::path::PathBuf;

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    all_bench_results::AllBenchResults, evaluator::Evaluators,
    feat::score_formatter::ScoreFormatter, init,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The benches to evaluate. Evaluates all benches by default.
    benches: Vec<String>,

    /// Only show results for the specified models
    #[arg(short, long)]
    models: Vec<String>,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Path to bench results
    #[arg(short, long, default_value = "results.ndjson")]
    results: PathBuf,
}

#[derive(Debug, thiserror::Error)]
#[error("an application error occurred")]
struct AppError;

#[tokio::main]
async fn main() -> Result<(), Report<AppError>> {
    init::init_tracing();
    dotenv().unwrap();
    let args = Args::parse();

    let responses = AllBenchResults::load(&args.results)
        .await
        .change_context(AppError)
        .attach("failed to load existing responses")?;

    let evaluators = Evaluators::default();
    // TODO: apply filters here
    // TODO: save/load results
    let scores = evaluators.score(responses);

    let formatter = ScoreFormatter::format(scores);

    formatter.print();

    Ok(())
}
