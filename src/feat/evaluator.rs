pub mod score;

use crate::feat::{
    bench::{AllBenchResults, BenchId},
    evaluator::score::{ScoredBench, Scores},
};
use linkme::distributed_slice;
use openrouter::completions::response::Choice;
pub use score::Score;
use std::collections::HashMap;

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
                scores.push(ScoredBench {
                    result: response,
                    score,
                });
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
