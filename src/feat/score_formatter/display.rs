use owo_colors::{
    OwoColorize,
    colors::{Cyan, Red, css::Gray},
};

use super::MARK_PASS;
use super::MARK_FAIL;
use super::aggregate::Totals;
use crate::feat::evaluator::score::ScoredBench;
use openrouter::completions::response::Choice;

pub(super) fn format_pass_fail(passed: bool) -> String {
    if passed {
        MARK_PASS.fg::<Cyan>().to_string()
    } else {
        MARK_FAIL.fg::<Red>().to_string()
    }
}

pub(super) fn format_passed(n_runs: usize, n_passed: usize, passed_all: bool) -> String {
    let n = if passed_all {
        format!("{n_passed}").fg::<Cyan>().to_string()
    } else {
        format!("{n_passed}").fg::<Red>().to_string()
    };
    format!("{}/{}", n, format!("{n_runs}").fg::<Cyan>())
}

pub(super) fn format_totals_passed(totals: &Totals) -> String {
    let passed = if totals.passed == totals.runs {
        format!("{}", totals.passed).fg::<Cyan>().to_string()
    } else {
        format!("{}", totals.passed).fg::<Red>().to_string()
    };
    format!("{}/{}", passed, totals.runs.to_string().fg::<Cyan>())
}

pub(super) fn print_response_details(
    scores: &[ScoredBench],
    div: &str,
    table_width: usize,
) {
    for (response_index, response) in scores.iter().enumerate() {
        let chat = get_chat_summary(response);
        let response_number = response_index + 1;
        let pass = response.score.passed;
        for (turn_index, message) in chat.iter().enumerate() {
            let message = message.content.replace('\n', " ");
            let rnumber = format_response_number(response_number, pass);
            let tnumber = format!("{}: ", turn_index + 1)
                .fg::<Gray>()
                .to_string();
            print!("{div}  {rnumber}{tnumber}");
            for (i, msg) in textwrap::wrap(&message, table_width - 13)
                .iter()
                .enumerate()
            {
                let msg = msg.fg::<Gray>();
                if i == 0 {
                    println!("{msg}");
                } else {
                    println!("{div}       {msg}");
                }
            }
        }
    }
}

pub(super) fn format_response_number(response_number: usize, pass: bool) -> String {
    let text = format!("{response_number}R");
    if pass {
        text.fg::<Cyan>().to_string()
    } else {
        text.fg::<Red>().to_string()
    }
}

fn get_chat_summary(response: &ScoredBench) -> Vec<ChatSummary> {
    response
        .result
        .responses
        .iter()
        .flat_map(|res| {
            res.choices.iter().map(|choice| match choice {
                Choice::NonStreaming(c) => ChatSummary {
                    _role: c.message.role.clone(),
                    content: c.message.content.clone().unwrap_or_default(),
                },
                _ => unimplemented!(),
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
struct ChatSummary {
    _role: String,
    content: String,
}

// ── Cost diff ───────────────────────────────────────────────────────────────

/// Compute percentage cost difference: `((agent_per_run - baseline_per_run) / baseline_per_run) * 100`
#[allow(clippy::cast_precision_loss)]
pub(super) fn pct_cost_diff(agent_per_run: f64, baseline_per_run: f64) -> f64 {
    (agent_per_run - baseline_per_run) / baseline_per_run * 100.0
}

pub(super) fn format_pct_cost_diff(pct: f64) -> String {
    format!("+{pct:.2}%")
}
