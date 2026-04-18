use ansi_width::ansi_width;
use owo_colors::OwoColorize;

use crate::{
    feat::cli::eval::SortColumn,
    feat::evaluator::score::Scores,
};

mod aggregate;
mod display;
mod row;

const MARK_PASS: &str = "✅ Pass";
const MARK_FAIL: &str = "❌ Fail";
const NCOLS: usize = 10;

use aggregate::{AggregatedScores, Totals};
use display::{
    format_pass_fail, format_passed, format_pct_cost_diff,
    format_totals_passed, pct_cost_diff, print_response_details,
};
use row::{ColWidths, Row};

#[derive(Debug, Clone)]
pub struct ScoreFormatter {
    scores: Scores,
    widths: ColWidths,
    agg: AggregatedScores,
}

impl ScoreFormatter {
    pub fn format(scores: Scores) -> Self {
        let agg = AggregatedScores::build(&scores);

        let mut bench_width = 4; // "bench"
        let mut agent_width = 5; // "agent"
        let mut model_width = 1;
        let mut in_tokens_width = 2;  // "in"
        let mut out_tokens_width = 3; // "out"
        let mut reason_width = 6;     // "reason"
        let mut cost_diff_width = "% cost diff  ".len();

        for (key, _) in &scores {
            let (base, agent) = split_agent_suffix(&key.bench_id.0);
            bench_width = bench_width.max(base.len());
            if let Some(agent) = agent {
                agent_width = agent_width.max(agent.len());
            }
            model_width = model_width.max(key.model_id.len());

            let entry = agg.for_key(key);
            in_tokens_width = in_tokens_width.max(digits(entry.prompt_tokens));
            out_tokens_width = out_tokens_width.max(digits(entry.completion_tokens));
            reason_width = reason_width.max(
                if entry.reasoning_tokens > 0 { digits(entry.reasoning_tokens) } else { 1 }
            );

            // Compute actual % cost diff string for width
            if agent.is_some() {
                if let Some(base_cpp) = agg.baseline_cost_per_run(base, &key.model_id.0) {
                    let cpp = entry.cost_per_run();
                    let pct = pct_cost_diff(cpp, base_cpp);
                    cost_diff_width = cost_diff_width.max(format_pct_cost_diff(pct).len());
                }
            }
        }

        let widths = ColWidths {
            bench: bench_width,
            agent: agent_width,
            model: model_width,
            result: ansi_width(MARK_PASS),
            passed: "passed".len() + 3,
            in_tokens: in_tokens_width,
            out_tokens: out_tokens_width,
            reason: reason_width,
            cost: "cost ($USD)    ".len(),
            cost_diff: cost_diff_width,
        };

        Self { scores, widths, agg }
    }

    #[allow(clippy::too_many_lines)]
    pub fn print(&self, sort_column: SortColumn, condensed: bool) {
        if self.scores.is_empty() {
            println!("no scores matching the filter criteria");
            return;
        }

        let sep = " | ";
        let div = " | ".fg::<owo_colors::colors::css::Gray>().to_string();

        println!("{}", Row::header(&self.widths).render(sep));
        let separator = Row::separator(&self.widths);
        println!("{}", separator.render(sep).fg::<owo_colors::colors::css::Gray>());
        let table_width = ansi_width(&separator.render(sep));

        // Sort entries by the requested column
        let mut entries: Vec<_> = self.scores.iter().collect();
        entries.sort_by(|(a, _), (b, _)| match sort_column {
            SortColumn::Bench => a.bench_id.cmp(&b.bench_id),
            SortColumn::Model => a.model_id.cmp(&b.model_id),
        });

        let mut totals = Totals::default();

        for (key, scores) in &entries {
            let entry = self.agg.for_key(key);
            let n_runs = entry.runs;
            let n_passed = entry.passed;
            totals.passed += n_passed;
            totals.runs += n_runs;
            let passed_all = n_passed == n_runs;

            let result_str = format_pass_fail(passed_all);
            let passed_str = format_passed(n_runs, n_passed, passed_all);
            let n_in = entry.prompt_tokens;
            let n_out = entry.completion_tokens;
            let n_reason = entry.reasoning_tokens;
            totals.prompt_tokens += n_in;
            totals.completion_tokens += n_out;
            totals.reasoning_tokens += n_reason;

            let cost = entry.cost;
            totals.cost += cost;

            let (base_bench, agent_suffix) = split_agent_suffix(&key.bench_id.0);
            let n_reason_str = if n_reason > 0 {
                n_reason.to_string()
            } else {
                "-".to_string()
            };

            // % cost diff: compare per-run cost against baseline (no-agent)
            let cost_diff_str = if agent_suffix.is_some() {
                self.agg
                    .baseline_cost_per_run(base_bench, &key.model_id.0)
                    .map_or_else(
                        || "-".to_string(),
                        |base_cpp| {
                            format_pct_cost_diff(pct_cost_diff(
                                entry.cost_per_run(),
                                base_cpp,
                            ))
                        },
                    )
            } else {
                "-".to_string()
            };

            let row = Row::new(
                [
                    base_bench.to_string().into(),
                    agent_suffix.unwrap_or("").to_string().into(),
                    key.model_id.to_string().into(),
                    result_str.into(),
                    passed_str.into(),
                    n_in.to_string().into(),
                    n_out.to_string().into(),
                    n_reason_str.into(),
                    format!("${cost:.9}").into(),
                    cost_diff_str.into(),
                ],
                &self.widths,
            );
            println!("{}", row.render(sep));

            // ── Response details ────────────────────────────────────────
            if !condensed {
                print_response_details(scores, &div, table_width);
            }
        }

        // ── Summary ─────────────────────────────────────────────────────

        let divider_line = format!(
            " {}",
            row::pad_str("", table_width - 2, '=').into_owned()
        );
        println!("{}", divider_line.fg::<owo_colors::colors::css::Gray>());

        println!("{}", Row::summary_header(&self.widths).render(sep));
        println!("{}", separator.render(sep).fg::<owo_colors::colors::css::Gray>());

        let pct_pass = format!("{:.2}%", totals.pct_pass() * 100.0);
        let cost_str = format!("${:.9}", totals.cost);
        let passed_str = format_totals_passed(&totals);
        let nreason_str = if totals.reasoning_tokens > 0 {
            totals.reasoning_tokens.to_string()
        } else {
            "-".to_string()
        };

        let summary = Row::new(
            [
                String::new().into(),
                String::new().into(),
                String::new().into(),
                pct_pass.into(),
                passed_str.into(),
                totals.prompt_tokens.to_string().into(),
                totals.completion_tokens.to_string().into(),
                nreason_str.into(),
                cost_str.into(),
                "-".into(),
            ],
            &self.widths,
        );
        println!("{}", summary.render(sep));
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────────

/// Number of decimal digits needed to represent `n`.
fn digits(n: u32) -> usize {
    n.checked_ilog10().map_or(1, |d| d as usize + 1)
}

/// Split `category/name+agent` into (`category/name`, Some(`agent`)).
/// If no `+`, returns (`id`, None).
fn split_agent_suffix(id: &str) -> (&str, Option<&str>) {
    match id.rsplit_once('+') {
        Some((base, agent)) => (base, Some(agent)),
        None => (id, None),
    }
}
