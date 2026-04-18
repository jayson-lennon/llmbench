pub mod score;

use openrouter::completions::response::Choice;
pub use score::Score;

/// The function that gets run to evaluate a response.
pub type EvaluatorFn = Box<dyn Fn(&[Choice]) -> Score>;

/// Runs an evaluator function for a specified bench.
pub struct Evaluator {
    pub bench: String,
    pub eval: EvaluatorFn,
}
