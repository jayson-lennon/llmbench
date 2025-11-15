use ansi_width::ansi_width;
use openrouter::completions::response::Choice;
use owo_colors::{
    OwoColorize,
    colors::{Cyan, Red, css::Gray},
};
use pad::PadStr;

const MARK_PASS: &str = "✅ Pass";
const MARK_FAIL: &str = "❌ Fail";
const HEADER_PASSES: &str = "passes";
const HEADER_OUTPUT_TOKENS: &str = "tokens";
const HEADER_COST: &str = "cost ($USD)    ";

use crate::evaluator::score::{ScoredResponse, Scores};

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

    /// Width of the "passes" column.
    passes_width: usize,

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
            if score.response.bench.len() > bench_width {
                bench_width = score.response.bench.len();
            }
            if score.response.request.model.len() > model_width {
                model_width = score.response.request.model.len();
            }
        }
        Self {
            scores,
            bench_width,
            model_width,
            result_width: ansi_width(MARK_PASS),
            passes_width: ansi_width(HEADER_PASSES),
            output_tokens_width: ansi_width(HEADER_OUTPUT_TOKENS),
            cost_width: ansi_width(HEADER_COST),
        }
    }

    pub fn print(&self) {
        let div = " | ".fg::<Gray>().to_string();
        self.print_header(&div);

        let table_width = self.print_line_under_header();

        // scores table
        {
            // summary line
            for (key, scores) in &self.scores {
                let n_runs = scores.len();
                let n_passes = get_number_of_passed_tests(scores);
                let passed_all = n_passes == n_runs;

                let result_str = get_formatted_pass_fail_section(passed_all);

                let passes_str = get_formatted_passes_section(n_runs, n_passes, passed_all);

                let n_tokens = get_number_of_tokens(scores);
                let cost = get_monetary_cost(scores);

                let bench = key.bench_id.to_string();
                let model = key.model_id.to_string();

                println!(
                    "{div}{bench}{div}{model}{div}{result_str}{div}{passes_str}{div}{n_tokens}{div}{cost}{div}",
                    bench = bench.pad_to_width(self.bench_width),
                    model = model.pad_to_width(self.model_width),
                    result_str = result_str.pad_to_width(self.result_width),
                    // HACK: pad_to_width can't handle a bunch of colors
                    passes_str = passes_str.pad_to_width(
                        passes_str.len() + {
                            if n_runs >= 10 && n_passes >= 10 {
                                // "10/10" needs +1
                                1
                            } else if n_runs >= 10 {
                                // "1/10" needs +2
                                2
                            } else {
                                // "1/2" needs +3
                                3
                            }
                        }
                    ),
                    n_tokens = n_tokens.to_string().pad_to_width(self.output_tokens_width),
                    // -1 for the dollar sign
                    cost = format_args!("${}", cost.to_string().pad_to_width(self.cost_width - 1))
                );

                // response summary
                {
                    for (response_index, response) in scores.iter().enumerate() {
                        let chat = get_chat_summary(response);

                        let response_number = response_index + 1;
                        let pass = response.score.pass;
                        // format individual messages
                        for message in chat {
                            let message = message.content.replace('\n', " ");
                            let wrapped_message = textwrap::wrap(&message, table_width - 13);
                            let rnumber_str = get_formatted_response_number(response_number, pass);
                            print!("{div}   {rnumber_str}: ",);
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
    }

    fn print_header(&self, div: &str) {
        {
            let header = format!(
                "{div}{bench}{div}{model}{div}{result}{div}{npasses}{div}{ntokens}{div}{cost}{div}",
                bench = "bench".pad_to_width(self.bench_width),
                model = "model".pad_to_width(self.model_width),
                result = "result".pad_to_width(self.result_width),
                npasses = HEADER_PASSES.pad_to_width(self.passes_width),
                ntokens = HEADER_OUTPUT_TOKENS.pad_to_width(self.output_tokens_width),
                cost = HEADER_COST.pad_to_width(self.cost_width)
            );
            println!("{header}",);
        }
    }

    fn print_line_under_header(&self) -> TableWidth {
        let plus = " + ";
        let line = format!(
            "{plus}{bench}{plus}{model}{plus}{result}{plus}{npasses}{plus}{ntokens}{plus}{cost}{plus}",
            bench = "".pad_to_width_with_char(self.bench_width, '-'),
            model = "".pad_to_width_with_char(self.model_width, '-'),
            result = "".pad_to_width_with_char(self.result_width, '-'),
            npasses = "".pad_to_width_with_char(self.passes_width, '-'),
            ntokens = "".pad_to_width_with_char(self.output_tokens_width, '-'),
            cost = "".pad_to_width_with_char(self.cost_width, '-'),
        );
        println!("{line}", line = line.fg::<Gray>());
        ansi_width(&line)
    }
}

fn get_formatted_response_number(response_number: usize, pass: bool) -> String {
    let text = format!("R{response_number}");
    if pass {
        text.fg::<Cyan>().to_string()
    } else {
        text.fg::<Red>().to_string()
    }
}

fn get_number_of_passed_tests(scores: &[ScoredResponse]) -> usize {
    scores.iter().fold(0, |pass, response| {
        let diff = usize::from(response.score.pass);
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

fn get_formatted_passes_section(n_runs: usize, n_passes: usize, passed_all: bool) -> String {
    let n_passes_str = if passed_all {
        format!("{n_passes}").fg::<Cyan>().to_string()
    } else {
        format!("{n_passes}").fg::<Red>().to_string()
    };
    let n_runs = format!("{n_runs}").fg::<Cyan>().to_string();
    format!("{n_passes_str}/{n_runs}")
}

fn get_chat_summary(response: &ScoredResponse) -> Vec<ChatSummary> {
    response
        .response
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

fn get_monetary_cost(scores: &[ScoredResponse]) -> f64 {
    scores
        .iter()
        .flat_map(|res| &res.response.responses)
        .fold(0.0, |cost, res| {
            cost + res
                .usage
                .as_ref()
                .map(|usage| usage.cost.unwrap_or_default())
                .unwrap_or_default()
        })
}

fn get_number_of_tokens(scores: &[ScoredResponse]) -> u32 {
    scores
        .iter()
        .flat_map(|res| &res.response.responses)
        .fold(0, |tokens, res| {
            tokens
                + res
                    .usage
                    .as_ref()
                    .map(|usage| usage.completion_tokens)
                    .unwrap_or_default()
        })
}

#[derive(Debug, Clone)]
struct ChatSummary {
    _role: String,
    content: String,
}
// | bench                                | model                      | result  | passes |
// + ------------------------------------ + -------------------------- + ------- + ------ +
// | decision_making/task_priority__naive | qwen/qwen3-235b-a22b:free  | ❌ Fail | 1/2
//  |   R1: Lorem ipsum dolor sit amet consectetur adipiscing elit. Sit amet consectetur
//          adipiscing elit quisque faucibus ex. Adipiscing elit quisque faucibus ex sapien.
//      R2: Vitae pellentesque sem placerat in id cursus mi. Cursus mi pretium tellus duis
//          convallis tempus leo. Tempus leo eu aenean sed diam urna tempor.
// | decision_making/task_priority__naive | qwen/qwen3-235b-a22b:free  | ❌ Fail | 1/2
//  |   R1: Lorem ipsum dolor sit amet consectetur adipiscing elit. Sit amet consectetur
//          adipiscing elit quisque faucibus ex. Adipiscing elit quisque faucibus ex sapien.
//      R2: Vitae pellentesque sem placerat in id cursus mi. Cursus mi pretium tellus duis
//          convallis tempus leo. Tempus leo eu aenean sed diam urna tempor.
// | decision_making/task_priority__naive | qwen/qwen3-235b-a22b:free  | ❌ Fail | 1/2
// | decision_making/task_priority__naive | qwen/qwen3-coder           | ❌ Fail | 0/1
