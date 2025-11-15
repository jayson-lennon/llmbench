use linkme::distributed_slice;
use openrouter::completions::response::Choice;

use crate::{
    bench_loader::BenchId,
    evaluator::{EVALUATORS, Evaluator, EvaluatorInit, Score, score::GetMessageExt},
};

#[distributed_slice(EVALUATORS)]
static EVALUATOR: EvaluatorInit = init;

fn init() -> Evaluator {
    Evaluator {
        bench: BenchId("decision_making/task_priority__recheck".to_string()),
        eval,
    }
}

fn eval(responses: &[Choice]) -> Score {
    match responses {
        [a, b] => match (a.get_message(), b.get_message()) {
            (Some(a), Some(b)) => {
                let pass_1 = a.to_lowercase() == "pick a game engine";
                let pass_2 = b.to_lowercase() == "pick a game engine";
                Score::builder().pass(pass_1 && pass_2).build()
            }
            _ => Score::fail(),
        },
        _ => Score::fail(),
    }
}
