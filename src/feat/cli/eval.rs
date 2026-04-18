use crate::feat::{
    bench::{AllBenchResults, bench_matches_pattern},
    cli::{CliError, SharedArgs, select_models},
    evaluator::Evaluators,
    model::ModelId,
    score_formatter::ScoreFormatter,
};
use clap::{Parser, ValueEnum};
use derive_more::Display;
use error_stack::{Report, ResultExt};

/// Sort column
#[derive(Copy, Debug, Default, Clone, ValueEnum, Display)]
pub enum SortColumn {
    /// Sort by bench
    #[default]
    #[display("bench")]
    Bench,
    /// Sort by model
    #[display("model")]
    Model,
}

/// Evaluate LLM responses
#[derive(Parser, Debug)]
pub struct EvalArgs {
    /// Evaluate the specified benches.
    #[arg(short, long)]
    benches: Vec<String>,

    /// Show results for the specified models.
    #[arg(short, long, group = "pickmodels")]
    models: Vec<ModelId>,

    /// Show results from groups defined in config.toml
    #[arg(short = 'g', long, group = "pickmodels")]
    model_groups: Vec<String>,

    /// Sort column
    #[arg(short, long, default_value_t = SortColumn::Bench)]
    sort: SortColumn,

    /// Suppress individual model responses in output
    #[arg(short, long)]
    condensed: bool,
}

pub async fn run(args: EvalArgs, shared_args: SharedArgs) -> Result<(), Report<[CliError]>> {
    let responses = AllBenchResults::load(&shared_args.results)
        .await
        .change_context(CliError)
        .attach("failed to load existing responses")?;

    let evaluators = Evaluators::default();

    // TODO: save/load results

    let mut scores = evaluators.score(responses);

    let model_filter = select_models(&args.models, &args.model_groups, &shared_args).await?;

    if !model_filter.is_empty() {
        scores = scores
            .into_iter()
            .filter(|(key, _)| {
                model_filter
                    .iter()
                    .any(|filter| key.model_id.contains(filter))
            })
            .collect();
    }

    let bench_filter = args.benches;

    if !bench_filter.is_empty() {
        scores = scores
            .into_iter()
            .filter(|(key, _)| {
                bench_filter
                    .iter()
                    .any(|pattern| bench_matches_pattern(&key.bench_id, pattern))
            })
            .collect();
    }

    let formatter = ScoreFormatter::format(scores);

    formatter.print(args.sort, args.condensed);

    Ok(())
}
