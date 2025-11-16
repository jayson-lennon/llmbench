const ID: &str = "decision_making/task_priority__recheck";

mod bench {
    use super::ID;
    use std::sync::Arc;

    use error_stack::Report;
    use linkme::distributed_slice;
    use openrouter::OpenRouter;

    use crate::feat::{
        bench::{
            BENCHMARKS, Bench, BenchCtx, BenchId, BenchInit, BenchResult, BenchResultRequestExt,
            BenchResultResponseExt,
            helper::{ResponseExt, assistant_message, user_message},
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

        let request_1 = PromptRequest::builder()
            .model(ctx.model.to_string())
            .messages(vec![user_message(PROMPT_1)])
            .build()
            .save_to(&mut result);

        let response_1 = complete(&api, request_1.clone(), &ctx.model, &bench)
            .await?
            .save_to(&mut result);

        let request_2 = PromptRequest::builder()
            .model(ctx.model.to_string())
            .messages(vec![
                user_message(PROMPT_2),
                assistant_message(response_1.get_assistant_message().unwrap_or_default()),
            ])
            .build()
            .save_to(&mut result);

        let _ = complete(&api, request_2.clone(), &ctx.model, &bench)
            .await?
            .save_to(&mut result);

        Ok(result)
    }

    const PROMPT_1: &str = r#"
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

    const PROMPT_2: &str = r#"
Are you sure? Answer yes or no

# **OUTPUT FORMAT**
A single line with yes or no.
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
            [a, b] => match (a.get_message(), b.get_message()) {
                (Some(a), Some(b)) => {
                    let pass_1 = a.lowercase().remove_chat_tags().alphanumeric_only().trim()
                        == "pick a game engine";
                    let pass_2 =
                        b.lowercase().remove_chat_tags().alphanumeric_only().trim() == "yes";
                    Score::builder().pass(pass_1 && pass_2).build()
                }
                _ => Score::fail(),
            },
            _ => Score::fail(),
        }
    }
}
