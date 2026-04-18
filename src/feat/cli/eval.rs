use std::{collections::HashMap, path::PathBuf};

use crate::feat::{
    bench::{AllBenchResults, BenchId, composable},
    cli::{CliError, SharedArgs, select_models},
    evaluator::Score,
    model::ModelId,
    score_formatter::ScoreFormatter,
};
use clap::{Parser, ValueEnum};
use derive_more::Display;
use error_stack::{Report, ResultExt};
use openrouter::completions::response::Choice;

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
    /// Evaluate the specified benches. Supports glob patterns.
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

    /// Agents.md file name or glob pattern from src/agents_md/ directory.
    #[arg(short, long)]
    agents: Option<String>,

    /// Hide bare (no agent) bench results when -a is specified.
    #[arg(long)]
    no_bare: bool,

    /// Override prompts directory (default: src/prompts/)
    #[arg(short, long)]
    prompts_dir: Option<PathBuf>,
}

pub async fn run(args: EvalArgs, shared_args: SharedArgs) -> Result<(), Report<[CliError]>> {
    let responses = AllBenchResults::load(&shared_args.results)
        .await
        .change_context(CliError)
        .attach("failed to load existing results")?;

    let prompts_dir = args
        .prompts_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("src/prompts"));
    let agents_dir = PathBuf::from("src/agents_md");

    // Discover benches and agents to build evaluators
    let patterns: Vec<&str> = if args.benches.is_empty() {
        vec!["*"]
    } else {
        args.benches.iter().map(String::as_str).collect()
    };

    let mut all_discovered = Vec::new();
    for pattern in &patterns {
        let discovered = composable::discover_benches(&prompts_dir, pattern)
            .change_context(CliError)
            .attach("failed to discover benches")?;
        all_discovered.extend(discovered);
    }
    all_discovered.sort_by(|a, b| a.id.cmp(&b.id));
    all_discovered.dedup_by(|a, b| a.id == b.id);

    let agents = match &args.agents {
        Some(pattern) => {
            composable::discover_agents(&agents_dir, pattern)
                .change_context(CliError)
                .attach("failed to discover agents.md files")?
        }
        None => vec![],
    };

    // Build evaluator map: bench_id -> eval_fn
    let evaluator_entries = composable::build_evaluators(&all_discovered, &agents);
    let evaluator_map: HashMap<BenchId, EvalFn> = evaluator_entries
        .into_iter()
        .map(|(bench_id, evaluator)| (bench_id, evaluator.eval))
        .collect();

    // Score responses
    let scores = score_responses(responses, &evaluator_map);

    // Filter by model
    let mut scores = scores;
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

    // Filter by bench pattern and agent
    //
    // When -a is specified:
    //   - Keep agent variants matching the agent pattern
    //   - Keep bare variants unless --no-bare
    //   - Both filtered by the bench pattern
    // When -a is not specified:
    //   - Keep everything matching bench pattern
    let bench_patterns: Vec<&str> = args.benches.iter().map(String::as_str).collect();
    let agent_pattern = args.agents.as_deref();
    let show_bare = !args.no_bare;

    scores = scores
        .into_iter()
        .filter(|(key, _)| {
            let full_id = &key.bench_id.0;
            let (base_bench, agent_suffix) = split_agent_suffix(full_id);

            // Filter by bench pattern
            if !bench_patterns.is_empty()
                && !bench_patterns
                    .iter()
                    .any(|p| matches_bench_id(base_bench, p))
            {
                return false;
            }

            // Filter by agent
            match (agent_pattern, agent_suffix) {
                // No -a flag: show everything
                (None, _) => true,
                // -a specified, this row is bare: show unless --no-bare
                (Some(_), None) => show_bare,
                // -a specified, this row has an agent: check it matches
                (Some(pat), Some(agent)) => matches_bench_id(agent, pat),
            }
        })
        .collect();

    let formatter = ScoreFormatter::format(scores);
    formatter.print(args.sort, args.condensed);

    Ok(())
}

fn matches_bench_id(id: &str, pattern: &str) -> bool {
    glob::Pattern::new(pattern).is_ok_and(|pat| pat.matches(id))
}

/// Split `category/name+agent` into (`category/name`, Some(`agent`)).
/// If no `+`, returns (`id`, None).
fn split_agent_suffix(id: &str) -> (&str, Option<&str>) {
    match id.rsplit_once('+') {
        Some((base, agent)) => (base, Some(agent)),
        None => (id, None),
    }
}

use crate::feat::evaluator::score::{ScoredBench, Scores};

type EvalFn = Box<dyn Fn(&[Choice]) -> Score>;

fn score_responses(
    responses: AllBenchResults,
    evaluator_map: &HashMap<BenchId, EvalFn>,
) -> Scores {
    let mut scored = Vec::new();

    for response in responses {
        if let Some(eval_fn) = evaluator_map.get(&response.bench) {
            let choices: Vec<Choice> = response
                .responses
                .iter()
                .flat_map(|res| res.choices.clone())
                .collect();

            let mut score = eval_fn(&choices);
            score.cost = response.responses.iter().fold(0.0, |cost, res| {
                res.usage
                    .as_ref()
                    .map_or(0.0, |usage| cost + usage.cost.unwrap_or_default())
            });
            score.completion_tokens = response.responses.iter().fold(0, |tokens, res| {
                res.usage
                    .as_ref()
                    .map_or(0, |usage| tokens + usage.completion_tokens)
            });
            score.prompt_tokens = response.responses.iter().fold(0, |tokens, res| {
                res.usage
                    .as_ref()
                    .map_or(0, |usage| tokens + usage.prompt_tokens)
            });
            score.reasoning_tokens = response.responses.iter().fold(0, |tokens, res| {
                res.usage
                    .as_ref()
                    .map_or(0, |usage| {
                        tokens
                            + usage
                                .completion_tokens_details
                                .as_ref()
                                .map_or(0, |d| d.reasoning_tokens)
                    })
            });

            scored.push(ScoredBench { result: response, score });
        }
    }

    Scores::from_iter(scored)
}
