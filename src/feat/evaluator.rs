//! Evaluation of LLM responses.
//!
//! Example implementation:
//! ```no_run
//! #[distributed_slice(EVALUATORS)]
//! static EVALUATOR: EvaluatorInit = init;
//!
//! fn init() -> Evaluator {
//!     Evaluator {
//!         bench: BenchId("bench_category/bench_dir_name".to_string()),
//!         eval,
//!     }
//! }
//! fn eval(responses: &[Choice]) -> Score {
//!     // your evaluation code here
//!     Score::pass()
//! }
//! ```
//!
//! To use the same evaluator with slightly changed prompts:
//!
//! ```no_run
//! init_benches!(eval, "bench_category/bench_name" => naive, superprompt, with_context);
//!
//! fn eval(evaluator: &Evaluator, content: &str) -> EvalResult {
//!     let pass = false; // perform evaluation
//!     EvalResult::builder()
//!         .bench(evaluator.bench.clone())
//!         .pass(pass)
//!         .build()
//! }
//! ```

pub mod score;

/// Associate multiple benchmarks with a single evaluator. For each bench listed, there should be a
/// corresponding directory of the same name in the `bench` dir.
///
/// This project uses the following as a baseline, but you can create whatever you want:
///
/// - **naive**: Basic "I want the answer and I want it now!" or "Just do this" prompt.
/// - **with_context**: Similar to naive, but provides reasoning as to why the prompt is being asked
///   in the first place. So "Here's what I'm working on, here's what I'm trying to accomplish, now
///   give me the answer".
/// - **wrong_leading**: Like _with_context_, but intentionally provides wrong information to try
///   and lead the LLM to the _wrong_ answer. This is to test if the LLM can still provide a
///   correct answer despite having wrong or superfluous information. Here is an example for a
///   cooking/recipe helper prompt:
///   - _(with_context)_: "I'm out of milk for my recipe that requires 1 cup, what should I do?"
///   - Expected: "Go to the grocery store"
///   - _(wrong_leading)_: "I'm out of milk for my recipe that requires 1 cup, but I was
///     thinking of going to watch a movie today. How much milk should I pour into the bowl?"
///   - Expected: "Go to the grocery store"
/// - **superprompt**: Detailed prompt tailored specifically to try and get the LLM to answer
///   correctly. "You are an expert at (thing) and your job is to (do this task) and then output
///   specifically this one thing etc. etc."
///
/// ```no_run
/// init_benches!(eval, "bench_category/bench_name" => naive, superprompt, with_context);
///
/// fn eval(evaluator: &Evaluator, content: &str) -> EvalResult {
///     let pass = false; // perform evaluation
///     EvalResult::builder()
///         .bench(evaluator.bench.clone())
///         .pass(pass)
///         .build()
/// }
/// ```
macro_rules! init_benches {
    ($func:ident, $bench_prefix:expr => $($name:ident),+ $(,)?) => {
        paste!{
        $(
            #[distributed_slice(EVALUATORS)]
            static [<EVALUATOR_ $name:upper>]: EvaluatorInit = [<init_ $name>];

            fn [<init_ $name>]() -> Evaluator {
                Evaluator {
                    bench: BenchId(format!("{}__{}", $bench_prefix, stringify!($name)).to_string()),
                    $func,
                }
            }
        )+
        }
    };
}

use linkme::distributed_slice;
use openrouter::completions::response::Choice;

use crate::feat::{
    bench::{AllBenchResults, BenchId},
    evaluator::score::{Score, ScoredResponse, Scores},
};
use std::collections::HashMap;

pub mod decision_making;

/// Evaluator modules must provide an `EvaluatorInit` function to include itself as an entry in the
/// [`Evaluators`] map.
pub type EvaluatorInit = fn() -> Evaluator;

/// The function that gets ran to evaluate a response.
pub type EvaluatorFn = fn(responses: &[Choice]) -> Score;

#[distributed_slice]
pub static EVALUATORS: [EvaluatorInit];

/// Runs an evaluator function for a specified bench.
#[derive(Debug, Clone)]
pub struct Evaluator {
    pub bench: BenchId,
    pub eval: EvaluatorFn,
}

#[derive(Debug)]
pub struct Evaluators {
    evaluators: HashMap<BenchId, Evaluator>,
}

impl Evaluators {
    pub fn score(&self, responses: AllBenchResults) -> Scores {
        let mut scores = Vec::new();

        for response in responses {
            let choices = response
                .responses
                .iter()
                // PERF: this is probably slow, but I want a nice API for the evaluators
                .flat_map(|res| res.choices.clone())
                .collect::<Vec<_>>();
            if let Some(evaluator) = self.evaluators.get(&response.bench) {
                let mut score = (evaluator.eval)(&choices);
                score.cost = response.responses.iter().fold(0.0, |cost, response| {
                    response
                        .usage
                        .as_ref()
                        .map_or(0.0, |usage| cost + usage.cost.unwrap_or_default())
                });
                score.completion_tokens = response.responses.iter().fold(0, |tokens, response| {
                    response
                        .usage
                        .as_ref()
                        .map_or(0, |usage| tokens + usage.completion_tokens)
                });
                scores.push(ScoredResponse { response, score });
            } else {
                tracing::error!(bench=%response.bench, "missing evaluator");
            }
        }

        Scores::from_iter(scores)
    }
}

impl Default for Evaluators {
    fn default() -> Self {
        Self {
            evaluators: EVALUATORS
                .iter()
                .map(|init| init())
                .map(|evaluator| (evaluator.bench.clone(), evaluator))
                .collect(),
        }
    }
}
