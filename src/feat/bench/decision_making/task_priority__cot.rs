const ID: &str = "decision_making/task_priority__cot";

mod bench {
    use super::ID;
    use crate::feat::bench::prelude::*;

    register_bench!(run);

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
I am working on a video game as a solo game developer. Given the following tasks for a video game development project for a city simulator, what item from the list below should I work on next in order to move the project forward in the most impactful way? Think through your answer step by step before responding.

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
    use crate::feat::bench::prelude::*;

    register_eval!(eval);

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
