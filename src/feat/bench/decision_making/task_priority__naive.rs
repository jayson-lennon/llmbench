const ID: &str = "decision_making/task_priority__naive";

mod bench {
    use super::ID;
    use std::sync::Arc;

    use error_stack::Report;
    use linkme::distributed_slice;
    use openrouter::OpenRouter;

    use crate::feat::{
        bench::{
            BENCHMARKS, Bench, BenchCtx, BenchId, BenchInit, BenchResult, helper::user_message,
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

        let request = PromptRequest::builder()
            .model(ctx.model.to_string())
            .messages(vec![user_message(PROMPT)])
            .build();

        let response = complete(&api, request.clone(), &ctx.model, &bench).await?;

        Ok(BenchResult {
            hash: ctx.run_hash,
            bench,
            model: ctx.model.clone(),
            requests: vec![request],
            responses: vec![response],
        })
    }

    const PROMPT: &str = r#"
Given the following tasks for a video game development project for a city simulator, what should be worked on next?

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
                    let answer = a.lowercase().alphanumeric_only().trim() == "pick a game engine";
                    Score::builder().pass(answer).build()
                }
                _ => Score::fail(),
            },
            _ => Score::fail(),
        }
    }
}
