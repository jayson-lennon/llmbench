use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    all_responses::AllResponses, bench_loader::BenchId, evaluator::Evaluators,
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

    // print steps
    // - iter all scores, find largest bench name + model name
    // - col1: len of longest bench (width)
    // - col2: len of longest model (width)
    // - response len = col1 + col2 + 4 (pass/fail) + 6 (padding)

    // decision_making/task_priority__naive | gemma-3b | ✅ Pass
    //    The output from the LLM that gets wrapped around etc
    // decision_making/task_priority__naive | gemma-3b |  ✅ Pass
    //    The output from the LLM
    // decision_making/task_priority__naive | gemma-3b |  ❌ Fail
    //    The output from the LLM
    // decision_making/task_priority__naive | gemma-3b |  ✅ Pass
    //    The output from the LLM
    // decision_making/task_priority__naive | gemma-3b |  ❌ Pass
    //    The output from the LLM
    // decision_making/task_priority__naive | gemma-3b |  ✅ Pass
    //    The output from the LLM
    // bench | model | result
    // decision/task_priority__naive | gemma | fail
    //
    // - list each model and their pass/fail + percent pass rate
    // - Model pass rate = (total passes / total tests) * 100
    // - list each benchmark and the sum of all model pass/fail + percent pass rate
    //

    Ok(())
}

fn get_bench_ids_to_run(args: &Args) -> Result<Vec<BenchId>, Report<AppError>> {
    let bench_ids = args
        .benches
        .iter()
        .map(|bench| BenchId::from_str(bench))
        .collect::<Vec<_>>();
    let mut errors = false;
    for id in &bench_ids {
        if let Err(e) = id {
            errors = true;
            tracing::error!(err=?e);
        }
    }
    if errors {
        return Err(Report::from(AppError)).attach("error parsing bench ids");
    }
    Ok(bench_ids.into_iter().flatten().collect::<Vec<_>>())
}
