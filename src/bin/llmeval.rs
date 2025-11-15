use std::path::PathBuf;

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    all_responses::AllResponses, evaluator::Evaluators, feat::score_formatter::ScoreFormatter, init,
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

    let responses = AllResponses::load(&args.results)
        .await
        .change_context(AppError)
        .attach("failed to load existing responses")?;

    let evaluators = Evaluators::default();
    let scores = evaluators.score(responses);
    dbg!(&scores);

    let formatter = ScoreFormatter::format(scores);

    formatter.print();

    // TODO: apply filters & sorting here

    Ok(())
}
