use linkme::distributed_slice;
use openrouter::completions::response::Choice;
use paste::paste;

use crate::{
    feat::bench::BenchId,
    feat::evaluator::{EVALUATORS, Evaluator, EvaluatorInit, Score, score::GetMessageExt},
};

init_benches!(eval, "decision_making/task_priority" => context, cot, naive, superprompt);

fn eval(responses: &[Choice]) -> Score {
    if let [choice] = responses
        && let Some(msg) = choice.get_message()
    {
        let pass = msg.to_lowercase() == "pick a game engine";
        return Score::builder().pass(pass).build();
    }
    Score::fail()
}
