use ansi_width::ansi_width;
use owo_colors::{
    OwoColorize,
    colors::{Cyan, Red, css::Gray},
};
use pad::PadStr;

const PASS: &str = "✅ Pass";
const FAIL: &str = "❌ Fail";

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
}

impl ScoreFormatter {
    pub fn format(scores: Scores) -> Self {
        let mut bench_width = 1;
        let mut model_width = 1;
        for score in &scores {
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
            result_width: ansi_width(PASS),
        }
    }

    pub fn print(&self) {
        let div = " | ".fg::<Gray>();
        // header
        {
            let header = format!(
                "{div}{bench}{div}{model}{div}{result}{div}",
                bench = "bench".pad_to_width(self.bench_width).bold(),
                model = "model".pad_to_width(self.model_width).bold(),
                result = "result".pad_to_width(self.result_width).bold(),
            );
            println!("{header}",);
        }

        // line under header
        {
            let plus = " + ";
            let line = format!(
                "{plus}{bench}{plus}{model}{plus}{result}{plus}",
                bench = "".pad_to_width_with_char(self.bench_width, '-'),
                model = "".pad_to_width_with_char(self.model_width, '-'),
                result = "".pad_to_width_with_char(self.result_width, '-'),
            );
            println!("{line}", line = line.fg::<Gray>());
        };

        for score in &self.scores {
            let result = if score.score.pass {
                PASS.fg::<Cyan>().to_string()
            } else {
                FAIL.fg::<Red>().to_string()
            };
            println!(
                "{div}{bench}{div}{model}{div}{result}{div}",
                bench = &score
                    .response
                    .bench
                    .to_string()
                    .pad_to_width(self.bench_width),
                model = &score
                    .response
                    .request
                    .model
                    .to_string()
                    .pad_to_width(self.model_width),
            );
        }
    }
}
