use std::{str::FromStr, time::Duration};

use crate::{
    error::{ErrContext, Suggestion},
    feat::{
        bench::{AllBenchResults, BENCHMARKS, BenchId, Benches},
        cli::{CliError, SharedArgs, select_models},
        completion::{self, PromptPayloadBatch, RunConfig},
        model::ModelId,
        persistence::{ResultWriterCmd, spawn_result_writer},
    },
};
use clap::Parser;
use error_stack::{Report, ResultExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task::JoinSet;

/// Run LLM benchmarks
#[derive(Parser, Debug)]
pub struct BenchArgs {
    /// The benches to run. Runs all benches by default.
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
    let api_key = {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            key
        } else {
            let Some(key) = args.api_key.clone() else {
                return Err(Report::from(CliError).expand())
                    .attach("missing API key")
                    .attach(Suggestion(
                        "create a .env file with an OPENROUTER_API_KEY variable",
                    ));
            };
            key
        }
    };

    let models = select_models(&args.models, &args.model_groups, &shared_args).await?;

    if models.is_empty() {
        return Err(Report::from(CliError).expand()).attach("no models selected");
    }

    let benches = {
        // make sure all input benches follow the correct format. bail otherwise
        let bench_ids = match get_bench_ids_to_run(&args) {
            Ok(ids) => ids,
            Err(er) => return Err(er),
        };

        let benches = BENCHMARKS.iter().map(|init| init()).collect::<Benches>();
        if bench_ids.is_empty() {
            benches
        } else {
            // bail if we cant find a bench that the user provided
            for id in &bench_ids {
                if !benches.contains(id) {
                    return Err(Report::from(CliError).expand())
                        .attach("unable to find bench")
                        .attach(ErrContext::new(format!("bench name='{id}'")))
                        .attach(Suggestion("make sure the bench exists"));
                }
            }
            benches
                .into_iter()
                .filter(|bench| bench_ids.contains(&bench.id))
                .collect()
        }
    };

    let existing_results = AllBenchResults::load(&shared_args.results)
        .await
        .change_context(CliError)
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
        multibar: MultiProgress::new(),
    };

    let mut joinset = JoinSet::new();

    let pb = config.multibar.add(ProgressBar::new(total_requests as u64));
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner} {msg} {bar:40.cyan/blue} {percent}% ({pos}/{len}) ETA: {eta} / Elapsed: {elapsed}",
        )
        .expect("programming error: invalid pb format string"),
    );
    pb.enable_steady_tick(Duration::from_millis(50));
    pb.set_message("Benching ");

    let result_writer = tokio::task::spawn(async move {
        spawn_result_writer(shared_args.results, total_requests, pb, rx).await;
    });

    for (model, requests) in requests {
        let config = config.clone();
        joinset.spawn(async move {
            completion::run(config, model, requests).await;
        });
    }

    tracing::trace!(count = joinset.len(), "tasks spawned");

    while (joinset.join_next().await).is_some() {
        tracing::trace!(remaining = joinset.len(), "task finished");
    }

    if let Err(e) = tx.send(ResultWriterCmd::Quit) {
        tracing::error!(err=?e, "failed to shutdown result writer task");
    }

    let _ = result_writer.await;
    Ok(())
}

fn get_bench_ids_to_run(args: &BenchArgs) -> Result<Vec<BenchId>, Report<[CliError]>> {
    let mut error: Option<Report<[CliError]>> = None;
    let (oks, errs): (Vec<_>, Vec<_>) = args
        .benches
        .iter()
        .map(|bench| BenchId::from_str(bench))
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
