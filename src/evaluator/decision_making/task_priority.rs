use linkme::distributed_slice;
use openrouter::completions::response::Choice;
use paste::paste;

use crate::{
    bench_loader::BenchId,
    evaluator::{EVALUATORS, Evaluator, EvaluatorInit, Score, score::GetMessageExt},
};

init_benches!(eval, "decision_making/task_priority" => naive, superprompt, with_context);

fn eval(responses: &[Choice]) -> Score {
    if let [choice] = responses
        && let Some(msg) = choice.get_message()
    {
        let pass = msg.to_lowercase() == "pick a game engine";
        return Score::builder().pass(pass).build();
    }
    Score::fail()
}
