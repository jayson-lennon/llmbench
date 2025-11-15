use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    all_bench_results::AllBenchResults,
    bench_loader::{BenchId, Benches},
    completion::{self, RunConfig},
    init,
    models::Models,
    promptrequest::PromptPayloadBatch,
    result_writer::{ResultWriterCmd, spawn_result_writer},
};
use tokio::task::JoinSet;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The benches to run. Runs all benches by default.
    benches: Vec<String>,

    /// Specify which models to bench. Runs on all models by default.
    #[arg(short, long)]
    models: Vec<String>,

    /// OpenRouter API key. (Prefer OPENROUTER_API_KEY env variable for security)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Number of runs per bench
    #[arg(long, default_value_t = 1)]
    n_runs: u32,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Path to bench dir
    #[arg(long, default_value = "bench")]
    bench_path: PathBuf,

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

    let api_key = {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            key
        } else {
            let Some(key) = args.api_key.clone() else {
                tracing::error!("OPENROUTER_API_KEY env variable or CLI arg required");
                return Err(Report::from(AppError));
            };
            key
        }
    };

    let benches = {
        // make sure all benches follow the correct format. bail otherwise
        let bench_ids = match get_bench_ids_to_run(&args) {
            Ok(ids) => ids,
            Err(er) => return Err(er),
        };

        let benches = Benches::new(args.bench_path)
            .await
            .change_context(AppError)
            .attach("failed to load benches")?;
        if bench_ids.is_empty() {
            benches
        } else {
            // bail if we cant find a bench that the user provided
            for id in &bench_ids {
                if !benches.contains(id) {
                    tracing::error!(id=%id, "bench not found");
                    return Err(Report::from(AppError));
                }
            }
            benches
                .into_iter()
                .filter(|bench| bench_ids.contains(&bench.id))
                .collect()
        }
    };

    let models = {
        let models = Models::load_from(args.config)
            .await
            .change_context(AppError)
            .attach("failed to load model list")?;
        if args.models.is_empty() {
            models
        } else {
            Models::from_iter(args.models)
        }
    };

    let existing_results = AllBenchResults::load(&args.results)
        .await
        .change_context(AppError)
        .attach("failed to load existing results")?;

    let mut requests = PromptPayloadBatch::new(models, &benches, args.n_runs);
    requests.filter_old_runs(existing_results);

    let requests = requests.split_by_models();

    let total_requests = requests
        .iter()
        .fold(0, |acc, (_, payloads)| acc + payloads.len());

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let config = RunConfig {
        api_key,
        results_tx: tx.clone(),
    };

    let mut set = JoinSet::new();

    let result_writer = tokio::task::spawn(async move {
        spawn_result_writer(args.results, rx).await;
    });

    for (model, requests) in requests {
        let config = config.clone();
        set.spawn(async move {
            completion::run(config, model, requests).await;
        });
    }

    tracing::trace!(count = set.len(), "tasks spawned");

    let mut completed = 0;
    while (set.join_next().await).is_some() {
        completed += 1;
        tracing::info!(
            remaining = (total_requests - completed),
            total = total_requests,
            "job status"
        );
    }

    tx.send(ResultWriterCmd::Quit).unwrap();

    let _ = result_writer.await;

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
