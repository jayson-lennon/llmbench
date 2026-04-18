use std::borrow::Cow;
use std::collections::HashMap;

use ansi_width::ansi_width;
use openrouter::completions::response::Choice;
use owo_colors::{
    OwoColorize,
    colors::{Cyan, Red, css::Gray},
};

use crate::{
    feat::cli::eval::SortColumn,
    feat::evaluator::score::{BenchModelKey, ScoredBench, Scores},
};

const MARK_PASS: &str = "✅ Pass";
const MARK_FAIL: &str = "❌ Fail";

// ── Column layout ───────────────────────────────────────────────────────────

/// Holds the computed widths for each column.
#[derive(Debug, Clone)]
struct ColWidths {
    bench: usize,
    agent: usize,
    model: usize,
    result: usize,
    passed: usize,
    in_tokens: usize,
    out_tokens: usize,
    reason: usize,
    cost: usize,
}

/// A single row of 9 pre-padded cell values.
struct Row([Cow<'static, str>; 9]);

impl Row {
    /// Build a row from raw values + column widths.
    fn new(
        cols: [Cow<'_, str>; 9],
        widths: &ColWidths,
    ) -> Self {
        let widths = [
            widths.bench,
            widths.agent,
            widths.model,
            widths.result,
            widths.passed,
            widths.in_tokens,
            widths.out_tokens,
            widths.reason,
            widths.cost,
        ];
        let cells: Vec<Cow<'static, str>> = cols
            .into_iter()
            .zip(widths)
            .map(|(val, w)| pad_str(&val, w, ' ').into_owned().into())
            .collect();
        let arr: [Cow<'static, str>; 9] = cells.try_into().unwrap_or_else(|v: Vec<_>| {
            panic!("expected 9 cells, got {}", v.len())
        });
        Self(arr)
    }

    /// Header labels.
    fn header(w: &ColWidths) -> Self {
        Self::new(
            [
                "bench".into(),
                "agent".into(),
                "model".into(),
                "result".into(),
                "passed".into(),
                "in".into(),
                "out".into(),
                "reason".into(),
                "cost ($USD)    ".into(),
            ],
            w,
        )
    }

    /// Summary header (% pass replaces result).
    fn summary_header(w: &ColWidths) -> Self {
        let mut row = Self::header(w);
        row.0[3] = pad_str("% pass", w.result, ' ').into_owned().into();
        row
    }

    /// Separator line of dashes.
    fn separator(w: &ColWidths) -> Self {
        let widths = [
            w.bench, w.agent, w.model, w.result, w.passed,
            w.in_tokens, w.out_tokens, w.reason, w.cost,
        ];
        let cells: Vec<Cow<'static, str>> = widths
            .iter()
            .map(|&width| pad_str("", width, '-').into_owned().into())
            .collect();
        Self(cells.try_into().unwrap())
    }

    /// Render the row with ` | ` dividers.
    fn render(&self, sep: &str) -> String {
        let mut out = String::from(sep);
        for cell in &self.0 {
            out.push_str(cell);
            out.push_str(sep);
        }
        out
    }
}

// ── Pre-computed aggregates ─────────────────────────────────────────────────

#[derive(Clone, Default, Debug)]
#[allow(clippy::struct_field_names)]
struct EntryAgg {
    prompt_tokens: u32,
    completion_tokens: u32,
    reasoning_tokens: u32,
    cost: f64,
    passed: usize,
    runs: usize,
}

#[derive(Default, Clone, Debug)]
struct AggregatedScores {
    entries: HashMap<BenchModelKey, EntryAgg>,
}

impl AggregatedScores {
    fn build(scores: &Scores) -> Self {
        let mut agg = Self::default();
        for score in scores.values().flatten() {
            agg.add(score);
        }
        agg
    }

    fn add(&mut self, bench: &ScoredBench) {
        let key = BenchModelKey {
            bench_id: bench.result.bench.clone(),
            model_id: bench.result.model.clone(),
        };
        let entry = self.entries.entry(key).or_default();
        entry.runs += 1;
        entry.passed += usize::from(bench.score.passed);
        for res in &bench.result.responses {
            if let Some(usage) = &res.usage {
                entry.prompt_tokens += usage.prompt_tokens;
                entry.completion_tokens += usage.completion_tokens;
                entry.reasoning_tokens += usage
                    .completion_tokens_details
                    .as_ref()
                    .map_or(0, |d| d.reasoning_tokens);
                entry.cost += usage.cost.unwrap_or_default();
            }
        }
    }

    fn for_key(&self, key: &BenchModelKey) -> &EntryAgg {
        static DEFAULT: EntryAgg = EntryAgg {
            prompt_tokens: 0,
            completion_tokens: 0,
            reasoning_tokens: 0,
            cost: 0.0,
            passed: 0,
            runs: 0,
        };
        self.entries.get(key).unwrap_or(&DEFAULT)
    }
}

// ── Formatter ───────────────────────────────────────────────────────────────

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
        };

        Self { scores, widths, agg }
    }

    pub fn print(&self, sort_column: SortColumn, condensed: bool) {
        if self.scores.is_empty() {
            println!("no scores matching the filter criteria");
            return;
        }

        let sep = " | ";
        let div = " | ".fg::<Gray>().to_string();

        println!("{}", Row::header(&self.widths).render(sep));
        let separator = Row::separator(&self.widths);
        println!("{}", separator.render(sep).fg::<Gray>());
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

        let divider_line = format!(" {}", pad_str("", table_width - 2, '=').into_owned());
        println!("{}", divider_line.fg::<Gray>());

        println!("{}", Row::summary_header(&self.widths).render(sep));
        println!("{}", separator.render(sep).fg::<Gray>());

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
            ],
            &self.widths,
        );
        println!("{}", summary.render(sep));
    }
}

// ── Display helpers ─────────────────────────────────────────────────────────

fn format_pass_fail(passed: bool) -> String {
    if passed {
        MARK_PASS.fg::<Cyan>().to_string()
    } else {
        MARK_FAIL.fg::<Red>().to_string()
    }
}

fn format_passed(n_runs: usize, n_passed: usize, passed_all: bool) -> String {
    let n = if passed_all {
        format!("{n_passed}").fg::<Cyan>().to_string()
    } else {
        format!("{n_passed}").fg::<Red>().to_string()
    };
    format!("{}/{}", n, format!("{n_runs}").fg::<Cyan>())
}

fn format_totals_passed(totals: &Totals) -> String {
    let passed = if totals.passed == totals.runs {
        format!("{}", totals.passed).fg::<Cyan>().to_string()
    } else {
        format!("{}", totals.passed).fg::<Red>().to_string()
    };
    format!("{}/{}", passed, totals.runs.to_string().fg::<Cyan>())
}

fn print_response_details(scores: &[ScoredBench], div: &str, table_width: usize) {
    for (response_index, response) in scores.iter().enumerate() {
        let chat = get_chat_summary(response);
        let response_number = response_index + 1;
        let pass = response.score.passed;
        for (turn_index, message) in chat.iter().enumerate() {
            let message = message.content.replace('\n', " ");
            let rnumber = format_response_number(response_number, pass);
            let tnumber = format!("{}: ", turn_index + 1).fg::<Gray>().to_string();
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

fn format_response_number(response_number: usize, pass: bool) -> String {
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

// ── Padding utilities ───────────────────────────────────────────────────────

fn pad_str(input: &str, amount: usize, ch: char) -> Cow<'_, str> {
    let visual_len = ansi_width(input);
    if visual_len < amount {
        let diff = amount - visual_len;
        let mut out = String::with_capacity(input.len() + diff);
        out.push_str(input);
        out.extend((0..diff).map(|_| ch));
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}

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

// ── Minor types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ChatSummary {
    _role: String,
    content: String,
}

#[derive(Debug, Clone, Default)]
struct Totals {
    passed: usize,
    runs: usize,
    prompt_tokens: u32,
    completion_tokens: u32,
    reasoning_tokens: u32,
    cost: f64,
}

impl Totals {
    #[allow(clippy::cast_precision_loss)]
    fn pct_pass(&self) -> f64 {
        self.passed as f64 / self.runs as f64
    }
}
