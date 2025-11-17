use std::borrow::Cow;

use ansi_width::ansi_width;
use openrouter::completions::response::Choice;
use owo_colors::{
    OwoColorize,
    colors::{Cyan, Red, css::Gray},
};

const MARK_PASS: &str = "✅ Pass";
const MARK_FAIL: &str = "❌ Fail";
const HEADER_BENCH: &str = "bench";
const HEADER_MODEL: &str = "model";
const HEADER_RESULT: &str = "result";
const HEADER_PASSED: &str = "passed";
const HEADER_OUTPUT_TOKENS: &str = "tokens";
const HEADER_COST: &str = "cost ($USD)    ";

const HEADER_SUMMARY_PCT_PASS: &str = "% pass";

use crate::{
    feat::cli::eval::SortColumn,
    feat::evaluator::score::{ScoredBench, Scores},
};

type TableWidth = usize;

#[derive(Debug, Clone)]
pub struct ScoreFormatter {
    scores: Scores,

    /// Width of the "bench name" column.
    bench_width: usize,

    /// Width of the "model name" column.
    model_width: usize,

    /// Width of the "result" column.
    result_width: usize,

    /// Width of the "passed" column.
    passed_width: usize,

    /// Width of the "output tokens" column.
    output_tokens_width: usize,

    /// Width of the "cost" column.
    cost_width: usize,
}

impl ScoreFormatter {
    pub fn format(scores: Scores) -> Self {
        let mut bench_width = 1;
        let mut model_width = 1;
        for score in scores.values().flatten() {
            if score.result.bench.len() > bench_width {
                bench_width = score.result.bench.len();
            }
            if score.result.model.len() > model_width {
                model_width = score.result.model.len();
            }
        }
        Self {
            scores,
            bench_width,
            model_width,
            result_width: ansi_width(MARK_PASS),
            passed_width: ansi_width(HEADER_PASSED) + 3,
            output_tokens_width: ansi_width(HEADER_OUTPUT_TOKENS) + 2,
            cost_width: ansi_width(HEADER_COST),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn print(&self, sort_column: SortColumn) {
        if self.scores.is_empty() {
            println!("no scores matching the filter criteria");
            return;
        }

        let div = " | ".fg::<Gray>().to_string();
        self.print_header(&div);

        let table_width = self.print_table_width_line();

        let mut scores = self.scores.iter().collect::<Vec<_>>();
        scores.sort_by(|(a, _), (b, _)| match sort_column {
            SortColumn::Bench => a.bench_id.cmp(&b.bench_id),
            SortColumn::Model => a.model_id.cmp(&b.model_id),
        });

        let mut totals = Totals::default();

        // scores table
        {
            // summary line
            for (key, scores) in &self.scores {
                let n_runs = scores.len();
                let n_passed = get_number_of_passed_tests(scores);
                {
                    totals.passed += n_passed;
                    totals.runs += n_runs;
                }
                let passed_all = n_passed == n_runs;

                let result_str = get_formatted_pass_fail_section(passed_all);

                let passed_str = get_formatted_passed_section(n_runs, n_passed, passed_all);

                let n_tokens = get_number_of_tokens(scores);
                {
                    totals.tokens += n_tokens;
                }
                let cost = get_monetary_cost(scores);
                {
                    totals.cost += cost;
                }

                let bench = key.bench_id.to_string();
                let model = key.model_id.to_string();

                let cost_str = format!("${cost:.9}");

                println!(
                    "{div}{bench}{div}{model}{div}{result_str}{div}{passed_str}{div}{n_tokens}{div}{cost}{div}",
                    bench = bench.pad(self.bench_width),
                    model = model.pad(self.model_width),
                    result_str = result_str.pad(self.result_width),
                    passed_str = passed_str.center(self.passed_width).pad(self.passed_width),
                    n_tokens = n_tokens.to_string().pad(self.output_tokens_width),
                    cost = cost_str.pad(self.cost_width)
                );

                // response summary
                {
                    for (response_index, response) in scores.iter().enumerate() {
                        let chat = get_chat_summary(response);

                        let response_number = response_index + 1;
                        let pass = response.score.passed;
                        // format individual messages
                        for (turn_index, message) in chat.iter().enumerate() {
                            let message = message.content.replace('\n', " ");
                            let rnumber_str = get_formatted_response_number(response_number, pass);
                            let tnumber_str =
                                format!("{}: ", turn_index + 1).fg::<Gray>().to_string();
                            print!("{div}  {rnumber_str}{tnumber_str}",);
                            let wrapped_message = textwrap::wrap(&message, table_width - 13);
                            for (i, msg) in wrapped_message.iter().enumerate() {
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
            }
        }

        // total summary
        {
            let width = self.table_width();
            let divider = format!(" {}", "".pad_with_char(width - 2, '='));

            println!("{divider}", divider = divider.fg::<Gray>());

            let header = format!(
                "{div}{bench}{div}{model}{div}{pct_pass}{div}{npassed}{div}{ntokens}{div}{cost}{div}",
                bench = "".pad(self.bench_width),
                model = "".pad(self.model_width),
                pct_pass = HEADER_SUMMARY_PCT_PASS.pad(self.result_width),
                npassed = HEADER_PASSED.pad(self.passed_width),
                ntokens = HEADER_OUTPUT_TOKENS.pad(self.output_tokens_width),
                cost = HEADER_COST.pad(self.cost_width)
            );
            println!("{header}");
            self.print_table_width_line();

            let pct_pass = format!("{:.2}%", (totals.pct_pass() * 100.0));
            let cost_str = format!("${:.9}", totals.cost);
            let passed_str = {
                let passed = if totals.passed == totals.runs {
                    format!("{}", totals.passed).fg::<Cyan>().to_string()
                } else {
                    format!("{}", totals.passed).fg::<Red>().to_string()
                };
                let passed_str = format!("{}/{}", passed, totals.runs.to_string().fg::<Cyan>());
                passed_str
            };
            let summary_line = format!(
                "{div}{bench}{div}{model}{div}{pct_pass}{div}{npassed}{div}{ntokens}{div}{cost}{div}",
                bench = "".pad(self.bench_width),
                model = "".pad(self.model_width),
                pct_pass = pct_pass.pad(self.result_width),
                npassed = passed_str.center(self.passed_width).pad(self.passed_width),
                ntokens = totals.tokens.to_string().pad(self.output_tokens_width),
                cost = cost_str.pad(self.cost_width)
            );
            println!("{summary_line}");
        }
    }

    fn print_header(&self, div: &str) {
        let header = format!(
            "{div}{bench}{div}{model}{div}{result}{div}{npassed}{div}{ntokens}{div}{cost}{div}",
            bench = HEADER_BENCH.pad(self.bench_width),
            model = HEADER_MODEL.pad(self.model_width),
            result = HEADER_RESULT.pad(self.result_width),
            npassed = HEADER_PASSED.pad(self.passed_width),
            ntokens = HEADER_OUTPUT_TOKENS.pad(self.output_tokens_width),
            cost = HEADER_COST.pad(self.cost_width)
        );
        println!("{header}",);
    }

    fn table_width(&self) -> TableWidth {
        let plus = " + ";
        let line = format!(
            "{plus}{bench}{plus}{model}{plus}{result}{plus}{npassed}{plus}{ntokens}{plus}{cost}{plus}",
            bench = "".pad_with_char(self.bench_width, '-'),
            model = "".pad_with_char(self.model_width, '-'),
            result = "".pad_with_char(self.result_width, '-'),
            npassed = "".pad_with_char(self.passed_width, '-'),
            ntokens = "".pad_with_char(self.output_tokens_width, '-'),
            cost = "".pad_with_char(self.cost_width, '-'),
        );
        ansi_width(&line)
    }

    fn print_table_width_line(&self) -> TableWidth {
        let plus = " + ";
        let line = format!(
            "{plus}{bench}{plus}{model}{plus}{result}{plus}{npassed}{plus}{ntokens}{plus}{cost}{plus}",
            bench = "".pad_with_char(self.bench_width, '-'),
            model = "".pad_with_char(self.model_width, '-'),
            result = "".pad_with_char(self.result_width, '-'),
            npassed = "".pad_with_char(self.passed_width, '-'),
            ntokens = "".pad_with_char(self.output_tokens_width, '-'),
            cost = "".pad_with_char(self.cost_width, '-'),
        );
        println!("{line}", line = line.fg::<Gray>());
        ansi_width(&line)
    }
}

fn get_formatted_response_number(response_number: usize, pass: bool) -> String {
    let text = format!("{response_number}R");
    if pass {
        text.fg::<Cyan>().to_string()
    } else {
        text.fg::<Red>().to_string()
    }
}

fn get_number_of_passed_tests(scores: &[ScoredBench]) -> usize {
    scores.iter().fold(0, |pass, response| {
        let diff = usize::from(response.score.passed);
        pass + diff
    })
}

fn get_formatted_pass_fail_section(passed_all: bool) -> String {
    if passed_all {
        MARK_PASS.fg::<Cyan>().to_string()
    } else {
        MARK_FAIL.fg::<Red>().to_string()
    }
}

fn get_formatted_passed_section(n_runs: usize, n_passed: usize, passed_all: bool) -> String {
    let n_passed_str = if passed_all {
        format!("{n_passed}").fg::<Cyan>().to_string()
    } else {
        format!("{n_passed}").fg::<Red>().to_string()
    };
    let n_runs = format!("{n_runs}").fg::<Cyan>().to_string();
    format!("{n_passed_str}/{n_runs}")
}

fn get_chat_summary(response: &ScoredBench) -> Vec<ChatSummary> {
    response
        .result
        .responses
        .iter()
        .flat_map(|res| {
            res.choices
                .iter()
                .map(|choice| match choice {
                    Choice::NonStreaming(choice) => ChatSummary {
                        _role: choice.message.role.clone(),
                        content: choice.message.content.clone().unwrap_or_default(),
                    },
                    _ => unimplemented!(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn get_monetary_cost(scores: &[ScoredBench]) -> f64 {
    scores
        .iter()
        .flat_map(|res| &res.result.responses)
        .fold(0.0, |cost, res| {
            cost + res
                .usage
                .as_ref()
                .map(|usage| usage.cost.unwrap_or_default())
                .unwrap_or_default()
        })
}

fn get_number_of_tokens(scores: &[ScoredBench]) -> u32 {
    scores
        .iter()
        .flat_map(|res| &res.result.responses)
        .fold(0, |tokens, res| {
            tokens
                + res
                    .usage
                    .as_ref()
                    .map(|usage| usage.completion_tokens)
                    .unwrap_or_default()
        })
}

fn pad_str(input: &str, amount: usize, ch: char) -> Cow<'_, str> {
    let visual_len = ansi_width(input);
    if visual_len < amount {
        let diff = amount - visual_len;
        let padding = (0..diff).map(|_| ch);
        let mut input = String::from(input);
        input.extend(padding);
        Cow::Owned(input)
    } else {
        Cow::Borrowed(input)
    }
}

/// Extension methods to make it easier to pad strings.
///
/// Note that this is ANSI color-code aware and operates on the visual representation of the text
/// in the terminal.
trait PadExt {
    fn pad(&self, len: usize) -> Cow<'_, str>;
    fn pad_with_char(&self, len: usize, ch: char) -> Cow<'_, str>;
    fn center(&self, width: usize) -> Cow<'_, str>;
}

impl PadExt for String {
    fn pad(&self, len: usize) -> Cow<'_, str> {
        pad_str(self.as_str(), len, ' ')
    }

    fn pad_with_char(&self, len: usize, ch: char) -> Cow<'_, str> {
        pad_str(self.as_str(), len, ch)
    }

    fn center(&self, width: usize) -> Cow<'_, str> {
        let text_size = ansi_width(self);
        if text_size < width.saturating_sub(1) {
            // |xx       | (9 wide, 2 len)
            // |   xx    | (9-2) = 7/2 = 3.5(trunc) = 3
            let padding = (width - text_size) / 2;
            let mut text = String::new();
            text.extend((0..padding).map(|_| ' '));
            text.push_str(self);
            text.extend((0..padding).map(|_| ' '));
            Cow::Owned(text)
        } else {
            Cow::Borrowed(self)
        }
    }
}

impl PadExt for Cow<'_, str> {
    fn pad(&self, len: usize) -> Cow<'_, str> {
        pad_str(self, len, ' ')
    }
    fn pad_with_char(&self, len: usize, ch: char) -> Cow<'_, str> {
        pad_str(self, len, ch)
    }

    fn center(&self, width: usize) -> Cow<'_, str> {
        let text_size = ansi_width(self);
        if text_size < width.saturating_sub(1) {
            // |xx       | (9 wide, 2 len)
            // |   xx    | (9-2) = 7/2 = 3.5(trunc) = 3
            let padding = (width - text_size) / 2;
            let mut text = String::new();
            text.extend((0..padding).map(|_| ' '));
            text.push_str(self);
            text.extend((0..padding).map(|_| ' '));
            Cow::Owned(text)
        } else {
            Cow::Borrowed(self)
        }
    }
}

impl PadExt for &'static str {
    fn pad(&self, len: usize) -> Cow<'_, str> {
        pad_str(self, len, ' ')
    }
    fn pad_with_char(&self, len: usize, ch: char) -> Cow<'_, str> {
        pad_str(self, len, ch)
    }

    fn center(&self, width: usize) -> Cow<'_, str> {
        let text_size = ansi_width(self);
        if text_size < width.saturating_sub(1) {
            // |xx       | (9 wide, 2 len)
            // |   xx    | (9-2) = 7/2 = 3.5(trunc) = 3
            let padding = (width - text_size) / 2;
            let mut text = String::new();
            text.extend((0..padding).map(|_| ' '));
            text.push_str(self);
            text.extend((0..padding).map(|_| ' '));
            Cow::Owned(text)
        } else {
            Cow::Borrowed(self)
        }
    }
}

#[derive(Debug, Clone)]
struct ChatSummary {
    _role: String,
    content: String,
}

#[derive(Debug, Clone, Default)]
struct Totals {
    passed: usize,
    runs: usize,
    tokens: u32,
    cost: f64,
}

impl Totals {
    #[allow(clippy::cast_precision_loss)]
    fn pct_pass(&self) -> f64 {
        self.passed as f64 / self.runs as f64
    }
}
