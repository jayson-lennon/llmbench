const ID: &str = "decision_making/task_priority__metaprompt";

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

        let prompt = {
            let metaprompt_input = PromptRequest::builder()
                .model(ctx.model.to_string())
                .messages(vec![user_message(METAPROMPT)])
                .build()
                .save_to(&mut result);
            complete(&api, metaprompt_input.clone(), &ctx.model, &bench)
                .await?
                .save_to(&mut result)
        };

        let prompt = format!(
            "{}\n{}",
            prompt.get_assistant_message().unwrap_or_default(),
            PROMPT
        );

        let request = PromptRequest::builder()
            .model(ctx.model.to_string())
            .messages(vec![user_message(prompt)])
            .build()
            .save_to(&mut result);

        let _ = complete(&api, request.clone(), &ctx.model, &bench)
            .await?
            .save_to(&mut result);

        Ok(result)
    }

    const METAPROMPT: &str = r#"
Task: Meta Prompting for In-Context Prompt Design
1. Input Analysis:
• Input: [User task instructions]
• Action: Analyze and extract key concepts, methodologies, challenges, and objectives.
2. Task Interpretation:
• Action: Synthesize the extracted information to define the core problem or task.
• Considerations: Identify constraints, goals, or requirements.
3. Prompt Design:
• Objective: Develop a structured prompt for problem-solving, including clear instructions, a step-by-step approach, and relevant background information.
4. Optional – Direct Solution Proposal:
• Objective: Propose initial steps or a complete solution strategy, ensuring feasibility and practicality.
5. Output Prompt: [Generate the output prompt]
Note: The output should be a coherent, actionable prompt or solution strategy tailored to the
specifics of the input task. Structure the prompt so it naturally ends by indicating that the user input will follow the end of the prompt. Do not include replacement indicators like [input here] or [paste here].

User task instructions:

You will be given a list of tasks for a solo video game developer to work on. Choose the most important task that should be completed next. The output format is the item by itself without additional commentary.
"#;

    const PROMPT: &str = r#"
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
    use crate::feat::bench::prelude::*;

    register_eval!(eval);

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
