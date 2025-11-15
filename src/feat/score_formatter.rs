use ansi_width::ansi_width;
use openrouter::completions::response::{self, Choice, NonStreamingChoice};
use owo_colors::{
    OwoColorize,
    colors::{Cyan, Red, css::Gray},
};
use pad::PadStr;

const MARK_PASS: &str = "✅ Pass";
const MARK_FAIL: &str = "❌ Fail";
const HEADER_PASSES: &str = "passes";

use crate::evaluator::score::Scores;

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
        }
    }

    pub fn print(&self) {
        let div = " | ".fg::<Gray>();
        // header
        {
            let header = format!(
                "{div}{bench}{div}{model}{div}{result}{div}{npasses}{div}",
                bench = "bench".pad_to_width(self.bench_width).bold(),
                model = "model".pad_to_width(self.model_width).bold(),
                result = "result".pad_to_width(self.result_width).bold(),
                npasses = "passes".pad_to_width(self.passes_width)
            );
            println!("{header}",);
        }

        // line under header
        let table_width = {
            let plus = " + ";
            let line = format!(
                "{plus}{bench}{plus}{model}{plus}{result}{plus}{npasses}{plus}",
                bench = "".pad_to_width_with_char(self.bench_width, '-'),
                model = "".pad_to_width_with_char(self.model_width, '-'),
                result = "".pad_to_width_with_char(self.result_width, '-'),
                npasses = "".pad_to_width_with_char(self.passes_width, '-'),
            );
            println!("{line}", line = line.fg::<Gray>());
            ansi_width(&line)
        };

        {
            // scores table
            for (key, scores) in &self.scores {
                let n_runs = scores.len();
                let n_passes = scores.iter().fold(0, |pass, response| {
                    let diff = if response.score.pass { 1 } else { 0 };
                    pass + diff
                });
                let passed_all = n_passes == n_runs;

                let result_str = if passed_all {
                    MARK_PASS.fg::<Cyan>().to_string()
                } else {
                    MARK_FAIL.fg::<Red>().to_string()
                };
                let n_passes_str = if passed_all {
                    format!("{n_passes}").fg::<Cyan>().to_string()
                } else {
                    format!("{n_passes}").fg::<Red>().to_string()
                };
                let n_runs_str = format!("{n_runs}").fg::<Cyan>().to_string();

                let bench = key.bench_id.to_string();
                let model = key.model_id.to_string();

                println!(
                    "{div}{bench}{div}{model}{div}{result_str}{div}{n_passes_str}/{n_runs_str}",
                    bench = bench.pad_to_width(self.bench_width),
                    model = model.pad_to_width(self.model_width),
                );

                // response summary
                {
                    for (response_index, response) in scores.iter().enumerate() {
                        let chat = response
                            .response
                            .responses
                            .iter()
                            .flat_map(|res| {
                                res.choices
                                    .iter()
                                    .map(|choice| match choice {
                                        Choice::NonStreaming(choice) => ChatSummary {
                                            role: choice.message.role.clone(),
                                            content: choice
                                                .message
                                                .content
                                                .clone()
                                                .unwrap_or_default(),
                                        },
                                        _ => unimplemented!(),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<_>>();

                        let response_number = response_index + 1;
                        let pass = response.score.pass;
                        // format individual messages
                        for message in chat {
                            let message = message.content.replace("\n", "");
                            let wrapped_message = textwrap::wrap(&message, table_width);
                            let rnumber_str = {
                                let text = format!("R{response_number}");
                                if pass {
                                    text.fg::<Cyan>().to_string()
                                } else {
                                    text.fg::<Red>().to_string()
                                }
                            };
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
}

#[derive(Debug, Clone)]
struct ChatSummary {
    role: String,
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
