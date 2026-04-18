use std::path::PathBuf;

use clap::Parser;
use error_stack::{Report, ResultExt};

use crate::feat::bench::{AllBenchResults, BenchId};
use crate::feat::cli::{CliError, SharedArgs};

/// Delete all results for a benchmark (all agents, all models).
#[derive(Parser, Debug)]
pub struct ResetArgs {
    /// Exact bench ID to delete (e.g. "decision_making/game_dev"). No globs.
    bench: BenchId,
}

pub async fn run(args: ResetArgs, shared_args: SharedArgs) -> Result<(), Report<[CliError]>> {
    let target_base = args.bench.0.clone();

    let all = AllBenchResults::load(&shared_args.results)
        .await
        .change_context(CliError)
        .attach("failed to load existing results")?;

    let before = all.inner.len();

    let kept: Vec<_> = all
        .inner
        .into_iter()
        .filter(|result| {
            let (base, _agent) = split_agent_suffix(&result.bench.0);
            base != target_base
        })
        .collect();

    let removed = before - kept.len();
    if removed == 0 {
        println!("no results found for bench '{target_base}'");
        return Ok(());
    }

    write_results(&shared_args.results, &kept)
        .await
        .change_context(CliError)
        .attach("failed to write results")?;

    println!("deleted {removed} result(s) for bench '{target_base}'");
    Ok(())
}

/// Split `category/name+agent` into (`category/name`, Some(`agent`)).
/// If no `+`, returns (`id`, None).
fn split_agent_suffix(id: &str) -> (&str, Option<&str>) {
    match id.rsplit_once('+') {
        Some((base, agent)) => (base, Some(agent)),
        None => (id, None),
    }
}

async fn write_results(
    path: &PathBuf,
    results: &[crate::feat::bench::BenchResult],
) -> Result<(), Report<CliError>> {
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await
        .change_context(CliError)
        .attach("failed to open results file for writing")?;

    for result in results {
        let line = serde_json::to_string(result)
            .change_context(CliError)
            .attach("failed to serialize result")?;
        file.write_all(line.as_bytes())
            .await
            .change_context(CliError)
            .attach("failed to write result")?;
        file.write_all(b"\n")
            .await
            .change_context(CliError)
            .attach("failed to write newline")?;
    }

    Ok(())
}
