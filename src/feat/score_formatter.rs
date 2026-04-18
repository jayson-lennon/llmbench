use ansi_width::ansi_width;
use owo_colors::{OwoColorize, colors::css::Gray};

use crate::{
    feat::cli::eval::SortColumn,
    feat::evaluator::score::Scores,
};

mod agent_summary;
mod aggregate;
mod display;
mod row;

const MARK_PASS: &str = "✅ Pass";
const MARK_FAIL: &str = "❌ Fail";
const NCOLS: usize = 10;

use aggregate::{AggregatedScores, Totals, split_agent_suffix};
use crate::feat::evaluator::score::BenchModelKey;
use agent_summary::AgentSummary;
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
        let mut agents_md_width = "AGENTS.md".len(); // 9
        let mut model_width = 1;
        let mut in_tokens_width = 2;  // "in"
        let mut out_tokens_width = 3; // "out"
        let mut reason_width = 6;     // "reason"
        let mut cost_diff_width = "med % cost Δ".len();

        let mut totals = Totals::default();

        for (key, _) in &scores {
            let (base, agent) = split_agent_suffix(&key.bench_id.0);
            bench_width = bench_width.max(base.len());
            if let Some(agent) = agent {
                agents_md_width = agents_md_width.max(agent.len());
            }
            model_width = model_width.max(key.model_id.len());

            let entry = agg.for_key(key);
            totals.prompt_tokens += entry.prompt_tokens;
            totals.completion_tokens += entry.completion_tokens;
            totals.reasoning_tokens += entry.reasoning_tokens;
            totals.cost += entry.cost;
            totals.runs += entry.runs;
            totals.passed += entry.passed;

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

        // Account for totals in column widths
        in_tokens_width = in_tokens_width.max(digits(totals.prompt_tokens));
        out_tokens_width = out_tokens_width.max(digits(totals.completion_tokens));
        reason_width = reason_width.max(
            if totals.reasoning_tokens > 0 { digits(totals.reasoning_tokens) } else { 1 }
        );

        // Account for aggregate labels in AGENTS.md column
        agents_md_width = agents_md_width
            .max("(baseline)".len())
            .max("Grand totals".len());

        let widths = ColWidths {
            model: model_width,
            bench: bench_width,
            agents_md: agents_md_width,
            result: ansi_width(MARK_PASS),
            passed: "passed".len() + 3,
            in_tokens: in_tokens_width,
            out_tokens: out_tokens_width,
            reason: reason_width,
            cost: "total cost ($USD)".len(),
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

        let div = " | ".fg::<Gray>().to_string();

        println!("{}", Row::header(&self.widths).render());
        let separator = Row::separator(&self.widths);
        println!("{}", separator.render_separator());
        let table_width = ansi_width(&separator.render());

        // Sort entries by the requested column
        let mut entries: Vec<_> = self.scores.iter().collect();
        entries.sort_by(|(a_key, _), (b_key, _)| {
            let a_entry = self.agg.for_key(a_key);
            let b_entry = self.agg.for_key(b_key);
            match sort_column {
                SortColumn::Bench => a_key.bench_id.cmp(&b_key.bench_id),
                SortColumn::Model => a_key.model_id.cmp(&b_key.model_id),
                SortColumn::Agent => {
                    let (_, agent_a) = split_agent_suffix(&a_key.bench_id.0);
                    let (_, agent_b) = split_agent_suffix(&b_key.bench_id.0);
                    agent_a.cmp(&agent_b).then_with(|| a_key.bench_id.cmp(&b_key.bench_id))
                }
                SortColumn::In => a_entry.prompt_tokens.cmp(&b_entry.prompt_tokens),
                SortColumn::Out => a_entry.completion_tokens.cmp(&b_entry.completion_tokens),
                SortColumn::Reason => a_entry.reasoning_tokens.cmp(&b_entry.reasoning_tokens),
                SortColumn::Cost => {
                    a_entry.cost_per_run()
                        .partial_cmp(&b_entry.cost_per_run())
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::CostDelta => {
                    let a_pct = self.cost_diff_pct(a_key);
                    let b_pct = self.cost_diff_pct(b_key);
                    a_pct.partial_cmp(&b_pct).unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        });

        let mut totals = Totals::default();
        let mut cost_diffs: Vec<f64> = Vec::new();

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
                            let pct = pct_cost_diff(entry.cost_per_run(), base_cpp);
                            cost_diffs.push(pct);
                            format_pct_cost_diff(pct)
                        },
                    )
            } else {
                "-".to_string()
            };

            let row = Row::new(
                [
                    key.model_id.to_string().into(),
                    base_bench.to_string().into(),
                    agent_suffix.unwrap_or("").to_string().into(),
                    result_str.into(),
                    passed_str.into(),
                    n_in.to_string().into(),
                    n_out.to_string().into(),
                    n_reason_str.into(),
                    format!("${:.9}", entry.cost_per_run()).into(),
                    cost_diff_str.into(),
                ],
                &self.widths,
            );
            println!("{}", row.render());

            // ── Response details ────────────────────────────────────────
            if !condensed {
                print_response_details(scores, &div, table_width);
            }
        }

        // ── Per-agent summary ────────────────────────────────────────────

        let divider_line = format!(
            " {}",
            row::pad_str("", table_width - 2, '=').into_owned()
        );
        println!("{}", divider_line.fg::<Gray>());

        AgentSummary::build(&self.agg).print(&self.widths);

        // ── Grand totals ────────────────────────────────────────────────

        println!("{}", divider_line.fg::<Gray>());

        println!("{}", Row::summary_header(&self.widths).render());
        println!("{}", separator.render_separator());

        let pct_pass = format!("{:.2}%", totals.pct_pass() * 100.0);
        let cost_str = format!("${:.9}", totals.cost);
        let passed_str = format_totals_passed(&totals);
        let nreason_str = if totals.reasoning_tokens > 0 {
            totals.reasoning_tokens.to_string()
        } else {
            "-".to_string()
        };

        let median_str = if cost_diffs.is_empty() {
            "-".to_string()
        } else {
            cost_diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = cost_diffs.len() / 2;
            let median = if cost_diffs.len().is_multiple_of(2) {
                f64::midpoint(cost_diffs[mid - 1], cost_diffs[mid])
            } else {
                cost_diffs[mid]
            };
            format_pct_cost_diff(median)
        };

        let summary = Row::new(
            [
                String::new().into(),
                String::new().into(),
                "Grand totals".into(),
                pct_pass.into(),
                passed_str.into(),
                totals.prompt_tokens.to_string().into(),
                totals.completion_tokens.to_string().into(),
                nreason_str.into(),
                cost_str.into(),
                median_str.into(),
            ],
            &self.widths,
        );
        println!("{}", summary.render());
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────────

/// Number of decimal digits needed to represent `n`.
fn digits(n: u32) -> usize {
    n.checked_ilog10().map_or(1, |d| d as usize + 1)
}

impl ScoreFormatter {
    /// Compute the % cost diff for a key, or `f64::NAN` if not applicable.
    fn cost_diff_pct(&self, key: &BenchModelKey) -> f64 {
        let (base, agent) = split_agent_suffix(&key.bench_id.0);
        if agent.is_none() {
            return f64::NAN;
        }
        let entry = self.agg.for_key(key);
        match self.agg.baseline_cost_per_run(base, &key.model_id.0) {
            Some(base_cpp) => pct_cost_diff(entry.cost_per_run(), base_cpp),
            None => f64::NAN,
        }
    }
}
