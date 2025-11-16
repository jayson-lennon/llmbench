use linkme::distributed_slice;
use openrouter::completions::response::Choice;

use crate::{
    feat::bench::BenchId,
    feat::evaluator::{EVALUATORS, Evaluator, EvaluatorInit, Score, score::GetMessageExt},
};

#[distributed_slice(EVALUATORS)]
static EVALUATOR: EvaluatorInit = init;

fn init() -> Evaluator {
    Evaluator {
        bench: BenchId("decision_making/task_priority__promptception".to_string()),
        eval,
    }
}

fn eval(responses: &[Choice]) -> Score {
    match responses {
        [a, b, ..] => match (a.get_message(), b.get_message()) {
            (Some(_), Some(b)) => {
                let answer = b.to_lowercase().trim() == "pick a game engine";
                Score::builder().pass(answer).build()
            }
            _ => Score::fail(),
        },
        _ => Score::fail(),
    }
}
