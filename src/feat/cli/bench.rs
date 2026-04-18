use std::{collections::HashMap, collections::VecDeque, str::FromStr, time::Duration};

use crate::{
    error::{ErrContext, Suggestion},
    feat::{
        bench::{AllBenchResults, BENCHMARKS, BenchId, Benches, bench_matches_pattern, contains_glob_chars},
        cli::{CliError, SharedArgs, select_models},
        completion::{self, PromptPayloadBatch, RunConfig, RunPayload},
        model::{ModelId, SelectedModels},
        persistence::{ResultWriterCmd, spawn_result_writer},
    },
};
use crate::feat::bench::BenchError;
use clap::Parser;
use error_stack::{Report, ResultExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task::JoinSet;

/// Run LLM benchmarks
#[derive(Parser, Debug)]
pub struct BenchArgs {
    /// The benches to run. Runs all benches by default.
    #[arg(short, long)]
    benches: Vec<String>,

    /// Bench specific models. Runs on all models by default.
    #[arg(short, long, group = "pickmodels")]
    models: Vec<ModelId>,

    /// Bench groups of models from groups defined config.toml
    #[arg(short = 'g', long, group = "pickmodels")]
    model_groups: Vec<String>,

    /// OpenRouter API key. (Prefer OPENROUTER_API_KEY env variable for security)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Number of runs per bench
    #[arg(long, default_value_t = 3)]
    n_runs: u32,
}

#[allow(clippy::missing_panics_doc)]
pub async fn run(args: BenchArgs, shared_args: SharedArgs) -> Result<(), Report<[CliError]>> {
    let api_key = get_api_key(&args)?;

    let models = select_and_validate_models(&args, &shared_args).await?;

    let benches = get_and_validate_benches(&args)?;

    let existing_results = load_existing_results(&shared_args).await?;

    let requests = prepare_model_requests(models, &benches, args.n_runs, existing_results);

    let (config, mut joinset, result_writer) =
        setup_progress_and_channels(api_key, &requests, &shared_args);

    spawn_completion_tasks(&config, requests, &mut joinset);

    wait_for_completion_and_shutdown(&mut joinset, config.results_tx.clone(), result_writer).await;
    Ok(())
}

/// Retrieves the OpenRouter API key from the environment variable `OPENROUTER_API_KEY`
/// or from the command-line arguments if the environment variable is not set.
/// Returns an error if neither is available.
fn get_api_key(args: &BenchArgs) -> Result<String, Report<[CliError]>> {
    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        Ok(key)
    } else {
        let Some(key) = args.api_key.clone() else {
            return Err(Report::from(CliError).expand())
                .attach("missing API key")
                .attach(Suggestion(
                    "create a .env file with an OPENROUTER_API_KEY variable",
                ));
        };
        Ok(key)
    }
}

/// Selects models for benchmarking based on the provided command-line arguments and shared configuration.
/// It resolves model IDs from individual models or model groups defined in the config file.
/// Returns the selected models or an error if no models are selected.
async fn select_and_validate_models(
    args: &BenchArgs,
    shared_args: &SharedArgs,
) -> Result<SelectedModels, Report<[CliError]>> {
    let models = select_models(&args.models, &args.model_groups, shared_args).await?;
    if models.is_empty() {
        return Err(Report::from(CliError).expand()).attach("no models selected");
    }
    Ok(models)
}

/// Parses and validates the benchmark IDs specified in the command-line arguments.
/// It loads all available benchmarks and filters them based on the provided IDs.
/// If no IDs are specified, returns all benchmarks.
/// Returns an error if any specified benchmark ID is invalid or not found.
fn get_and_validate_benches(args: &BenchArgs) -> Result<Benches, Report<[CliError]>> {
    let patterns = get_bench_patterns(args)?;
    let benches = BENCHMARKS.iter().map(|init| init()).collect::<Benches>();
    if patterns.is_empty() {
        Ok(benches)
    } else {
        let matched: Benches = benches
            .iter()
            .filter(|bench| {
                patterns
                    .iter()
                    .any(|pattern| bench_matches_pattern(&bench.id, pattern))
            })
            .cloned()
            .collect();
        if matched.benches.is_empty() {
            return Err(Report::from(CliError).expand())
                .attach("no benches matched the given patterns")
                .attach_with(|| ErrContext::new(format!("patterns={:?}", patterns)))
                .attach(Suggestion("make sure the bench exists"));
        }
        Ok(matched)
    }
}

/// Loads existing benchmark results from the file specified in the shared arguments.
/// This is used to filter out already completed runs to avoid duplication.
/// Returns the loaded results or an error if loading fails.
async fn load_existing_results(
    shared_args: &SharedArgs,
) -> Result<AllBenchResults, Report<CliError>> {
    AllBenchResults::load(&shared_args.results)
        .await
        .change_context(CliError)
        .attach("failed to load existing results")
}

/// Creates request payloads for the specified models and benchmarks, generating the given number of runs per combination.
/// Filters out requests that have already been completed based on the existing results.
/// Returns a map of model IDs to their corresponding request queues, grouped by model for concurrent processing.
fn prepare_model_requests(
    models: SelectedModels,
    benches: &Benches,
    n_runs: u32,
    existing_results: AllBenchResults,
) -> HashMap<ModelId, VecDeque<RunPayload>> {
    let mut requests = PromptPayloadBatch::new(models, benches, n_runs);
    requests.filter_old_runs(existing_results);
    requests.split_by_models()
}

/// Sets up the progress bar, communication channels, and result writer task for the benchmarking process.
/// Creates a progress bar displaying the total number of requests and spawns a background task to write results.
/// Returns the run configuration, an empty task join set, and the result writer task handle.
fn setup_progress_and_channels(
    api_key: String,
    requests: &HashMap<ModelId, VecDeque<RunPayload>>,
    shared_args: &SharedArgs,
) -> (RunConfig, JoinSet<()>, tokio::task::JoinHandle<()>) {
    let total_requests = requests
        .iter()
        .fold(0, |acc, (_, payloads)| acc + payloads.len());

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let config = RunConfig {
        api_key,
        results_tx: tx.clone(),
        multibar: MultiProgress::new(),
    };

    let joinset = JoinSet::new();

    let pb = config.multibar.add(ProgressBar::new(total_requests as u64));
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner} {msg} {bar:40.cyan/blue} {percent}% ({pos}/{len}) ETA: {eta} / Elapsed: {elapsed}",
        )
        .expect("programming error: invalid pb format string"),
    );
    pb.enable_steady_tick(Duration::from_millis(50));
    pb.set_message("Benching ");

    let results_path = shared_args.results.clone();
    let result_writer = tokio::task::spawn(async move {
        spawn_result_writer(results_path, total_requests, pb, rx).await;
    });

    (config, joinset, result_writer)
}

/// Spawns asynchronous tasks to run benchmarks for each model using the provided configuration.
/// Each model gets its own task with its queue of requests, allowing concurrent processing across models.
/// The tasks are added to the provided join set for later synchronization.
fn spawn_completion_tasks(
    config: &RunConfig,
    requests: HashMap<ModelId, VecDeque<RunPayload>>,
    joinset: &mut JoinSet<()>,
) {
    for (model, requests) in requests {
        let config = config.clone();
        joinset.spawn(async move {
            completion::run(config, model, requests).await;
        });
    }
}

/// Waits for all spawned benchmark tasks to complete, logging progress along the way.
/// After all tasks finish, sends a quit signal to the result writer and waits for it to shut down cleanly.
/// This ensures all results are written before the function returns.
async fn wait_for_completion_and_shutdown(
    joinset: &mut JoinSet<()>,
    tx: tokio::sync::mpsc::UnboundedSender<ResultWriterCmd>,
    result_writer: tokio::task::JoinHandle<()>,
) {
    tracing::trace!(count = joinset.len(), "tasks spawned");

    while (joinset.join_next().await).is_some() {
        tracing::trace!(remaining = joinset.len(), "task finished");
    }

    if let Err(e) = tx.send(ResultWriterCmd::Quit) {
        tracing::error!(err=?e, "failed to shutdown result writer task");
    }

    let _ = result_writer.await;
}

fn get_bench_patterns(args: &BenchArgs) -> Result<Vec<String>, Report<[CliError]>> {
    let mut error: Option<Report<[CliError]>> = None;
    let (oks, errs): (Vec<_>, Vec<_>) = args
        .benches
        .iter()
        .map(|input| validate_bench_pattern(input))
        .partition(Result::is_ok);

    for entry in errs {
        let err = entry.unwrap_err();
        if let Some(error) = error.as_mut() {
            error.push(err.change_context(CliError));
        } else {
            error = Some(err.change_context(CliError).expand());
        }
    }

    if let Some(error) = error {
        Err(error)
    } else {
        Ok(oks.into_iter().flatten().collect::<Vec<_>>())
    }
}

/// Validate a bench pattern. Non-glob patterns must follow `category/benchname` format.
/// Glob patterns are validated by the `glob` crate.
fn validate_bench_pattern(input: &str) -> Result<String, Report<BenchError>> {
    if contains_glob_chars(input) {
        glob::Pattern::new(input)
            .map(|_| input.to_string())
            .map_err(|e| {
                Report::from(BenchError)
                    .attach("invalid glob pattern")
                    .attach(ErrContext::new(format!("input '{input}'")))
                    .attach(ErrContext::new(format!("error: {e}")))
                    .attach(Suggestion(
                        "check glob syntax: * matches any characters, ? matches one character",
                    ))
            })
    } else {
        BenchId::from_str(input).map(|_| input.to_string())
    }
}
