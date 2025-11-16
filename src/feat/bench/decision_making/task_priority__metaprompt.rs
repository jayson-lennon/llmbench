const ID: &str = "decision_making/task_priority__metaprompt";

mod bench {
    use super::ID;
    use std::sync::Arc;

    use error_stack::Report;
    use linkme::distributed_slice;
    use openrouter::OpenRouter;

    use crate::feat::{
        bench::{
            BENCHMARKS, Bench, BenchCtx, BenchId, BenchInit, BenchResult,
            helper::{ResponseExt, user_message},
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

        let prompt = {
            let metaprompt_input = PromptRequest::builder()
                .model(ctx.model.to_string())
                .messages(vec![user_message(METAPROMPT)])
                .build();
            result.requests.push(metaprompt_input.clone());
            complete(&api, metaprompt_input.clone(), &ctx.model, &bench).await?
        };
        result.responses.push(prompt.clone());

        let prompt = format!(
            "{}\n{}",
            prompt.get_assistant_message().unwrap_or_default(),
            PROMPT_1
        );

        let request = PromptRequest::builder()
            .model(ctx.model.to_string())
            .messages(vec![user_message(prompt)])
            .build();

        result.requests.push(request.clone());
        let response = complete(&api, request.clone(), &ctx.model, &bench).await?;
        result.responses.push(response);

        Ok(result)
    }

    const METAPROMPT: &str = r#"
You are an expert Prompt Engineer specializing in creating effective, concise, and high-performing prompts for large language models. Your goal is to craft prompts that maximize clarity, creativity, and output quality while minimizing hallucinations or off-topic responses.

When given a task description, follow these steps to generate an optimized prompt:

1. **Understand the Task**: Analyze the core objective, key constraints, desired output format (e.g., list, table, step-by-step), tone (e.g., professional, fun), and any specific requirements (e.g., length, examples). The task to perform will not include the data to work on - that data will be substituted by the user by appending at the end of your generated prompt.

2. **Enhance with Best Practices**:
   - Use role-playing (e.g., "You are an expert in [field]").
   - Include clear instructions with examples if helpful.
   - Specify output structure using Markdown (e.g., bullet points, tables).
   - Add chain-of-thought reasoning if the task benefits from step-by-step thinking.
   - If the user includes an output format, emphasize the format in the output prompt

3. **Output Format**:
   - The full optimized prompt without any additional commentary
   - _Do not_ mention anything about the user appending additional prompt data.

## Task Description

You will be given a list of tasks for a solo video game developer to work on. Choose the most important task that should be completed next. The output format is the item by itself without additional commentary.
"#;

    const PROMPT_1: &str = r#"
---
** Task List **

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
            [_, answer] => match answer.get_message() {
                Some(answer) => {
                    let pass = answer
                        .lowercase()
                        .remove_chat_tags()
                        .alphanumeric_only()
                        .trim()
                        == "pick a game engine";
                    Score::builder().pass(pass).build()
                }
                _ => Score::fail(),
            },
            _ => Score::fail(),
        }
    }
}
