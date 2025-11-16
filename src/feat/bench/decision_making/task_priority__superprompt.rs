const ID: &str = "decision_making/task_priority__superprompt";

mod bench {
    use super::ID;
    use std::sync::Arc;

    use error_stack::Report;
    use linkme::distributed_slice;
    use openrouter::OpenRouter;

    use crate::feat::{
        bench::{
            BENCHMARKS, Bench, BenchCtx, BenchId, BenchInit, BenchResult, BenchResultRequestExt,
            BenchResultResponseExt, helper::user_message,
        },
        completion::{
            PromptRequest,
            worker::{CompletionError, complete},
        },
    };

    #[distributed_slice(BENCHMARKS)]
    static BENCHMARK: BenchInit = init;

    fn init() -> Bench {
        Bench::new(BenchId(ID.to_string()), run)
    }

    async fn run(
        api: Arc<OpenRouter>,
        ctx: BenchCtx,
    ) -> Result<BenchResult, Report<CompletionError>> {
        let bench = BenchId(ID.to_string());
        let mut result = BenchResult {
            hash: ctx.run_hash,
            bench: bench.clone(),
            model: ctx.model.clone(),
            requests: vec![],
            responses: vec![],
        };

        let request = PromptRequest::builder()
            .model(ctx.model.to_string())
            .messages(vec![user_message(PROMPT)])
            .build()
            .save_to(&mut result);

        let _ = complete(&api, request.clone(), &ctx.model, &bench)
            .await?
            .save_to(&mut result);

        Ok(result)
    }

    const PROMPT: &str = r#"
You are an expert game developer and project manager. Your job is to determine the task that should be completed next given a list of unordered tasks.

There is only 1 developer working on the game, so the task should provide the most value in driving the project forward. The most important task in a game development project is one that is blocking other incomplete tasks.

Think about your answer step by step before responding.

## Building Graphics
- [x] Residential zone
- [ ] Commercial zone
- [x] Industrial zone
- [ ] Utilities
- [x] Emergency Response

## Sound Effects
- [ ] Ambient city noise
- [ ] UI

## Technical Implementation
- [ ] Population monitor
- [ ] Loan management
- [ ] Pick a game engine
- [ ] Zone grid placement
- [ ] Calculating revenue based on zone capacity and usage levels

## Game Design
- [ ] Loss conditions
  - [x] Out of money
  - [ ] City destroyed
  - [ ] Unhappy residents
- [ ] Win conditions
- [ ] Keyboard shortcuts
- [x] Advisors
- [x] Road building algorithm
- [x] Weather transitions
- [ ] Available zones and buildings

## Animations
- [ ] People walking
- [ ] Cars driving
- [ ] Title screen

# **OUTPUT FORMAT**
A single line with the task to work on. No additional commentary.
    "#;
}

mod eval {
    use super::ID;
    use linkme::distributed_slice;
    use openrouter::completions::response::Choice;

    use crate::feat::{
        bench::{BenchId, helper::StringBenchExt},
        evaluator::{EVALUATORS, Evaluator, EvaluatorInit, Score, score::GetMessageExt},
    };

    #[distributed_slice(EVALUATORS)]
    static EVALUATOR: EvaluatorInit = init;

    fn init() -> Evaluator {
        Evaluator {
            bench: BenchId(ID.to_string()),
            eval,
        }
    }

    fn eval(responses: &[Choice]) -> Score {
        match responses {
            [a] => match a.get_message() {
                Some(a) => {
                    let answer = a.lowercase().remove_chat_tags().alphanumeric_only().trim()
                        == "pick a game engine";
                    Score::builder().pass(answer).build()
                }
                _ => Score::fail(),
            },
            _ => Score::fail(),
        }
    }
}
