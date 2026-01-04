//! This is just a fun test to see what happens if we try to summarize a summary.
//!
//! Re-enable this module if you want to try it out

const ID: &str = "summary/spam";

mod bench {
    use tokio::{fs::OpenOptions, io::AsyncWriteExt};

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

        let mut i = 1;
        let mut prompt = format!("{SUMMARIZE}{INITIAL_CONTENT}");
        while i <= 100 {
            let request = PromptRequest::builder()
                .model(ctx.model.to_string())
                .messages(vec![user_message(prompt)])
                .build()
                .save_to(&mut result);

            let response = complete(&api, request.clone(), &ctx.model, &bench)
                .await?
                .save_to(&mut result);

            if let Some(summary) = response.get_assistant_message() {
                let mut out_file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("summary-spam.txt")
                    .await;
                match out_file {
                    Ok(ref mut out_file) => {
                        let output_to_write = format!(
                            "\n\n=========================================\nSUMMARY n={i}\n=========================================\n{summary}"
                        );
                        if let Err(e) = out_file.write_all(output_to_write.as_bytes()).await {
                            tracing::error!(err=?e, "failed to write output for summary spam");
                        }
                    }
                    Err(e) => {
                        tracing::error!(err=?e, "failed to open file to output summary spam");
                    }
                }
                prompt = format!("{SUMMARIZE}{summary}");
                tracing::debug!(prompt = prompt);
            } else {
                tracing::error!(iterations = i, "summary spam aborted early");
                break;
            }

            i += 1;
        }

        Ok(result)
    }

    const SUMMARIZE: &str = r#"Summarize the following news article:

    "#;

    const INITIAL_CONTENT: &str = r#"
# The Dawn of a New Era: Large Language Models Reshape the World

**By Elena Vasquez**  
*Tech Correspondent*  
**New York | October 15, 2024**

In a quiet server farm outside Seattle, millions of lines of code hum to life every second, processing queries from students cramming for exams, lawyers drafting contracts, and artists brainstorming their next masterpiece. This is the invisible engine of the Large Language Models (LLMs)—the AI powerhouses that have exploded onto the global stage in recent years, transforming everything from daily productivity to geopolitical strategy.

Once the stuff of science fiction, LLMs like OpenAI's GPT-4o, Google's Gemini 2.0, Anthropic's Claude 3.5 Sonnet, and xAI's Grok have become ubiquitous. Capable of generating human-like text, code, images, and even music, these models are trained on vast datasets scraped from the internet, books, and code repositories. With parameters numbering in the trillions—GPT-4 is estimated at 1.76 trillion—their ability to predict and generate the next word in a sequence has unlocked unprecedented capabilities.

But as LLMs permeate society, questions loom large: Are they a boon for innovation or a ticking time bomb of misinformation, job loss, and existential risk? This article delves into the meteoric rise of LLMs, their profound impacts, and the urgent debates surrounding their future.

## From Chatbots to Cognitive Revolution

The story of LLMs traces back to the 2017 paper "Attention is All You Need," which introduced the Transformer architecture—the backbone of modern AI. Pioneered by researchers at Google, this innovation allowed models to weigh the importance of different words in a sentence, enabling far more coherent language generation.

Fast-forward to 2022, when OpenAI's ChatGPT burst into public consciousness. Launched as a free research preview, it amassed 100 million users in just two months, shattering records previously held by apps like TikTok. "It was like handing the world a magic typewriter," says Dr. Fei-Fei Li, co-director of Stanford's Human-Centered AI Institute. "Suddenly, anyone could converse with a machine that felt alive."

Today, the LLM landscape is a high-stakes arms race. OpenAI's recent release of o1-preview, a "reasoning model" that thinks step-by-step like a human before responding, scored 83% on the International Math Olympiad qualifying exam—surpassing most human contestants. Google's Gemini 2.0, integrated into Android devices, handles multimodal inputs like voice, image, and text seamlessly. Meanwhile, Meta's Llama 3.1, released open-source, has empowered startups and researchers worldwide to build custom AIs without billionaire backing.

Investment pours in: Venture capital in AI startups hit $50 billion in 2023, with LLMs at the core. NVIDIA's stock has surged 800% since 2022, fueled by demand for the GPUs that train these behemoths.

## Powering Industries: Real-World Transformations

LLMs aren't confined to chat interfaces; they're embedded in the fabric of industries.

**Healthcare:** At Mayo Clinic, an LLM-powered tool analyzes patient notes to suggest diagnoses, reducing errors by 30%, according to a 2024 study in *The Lancet*. IBM's Watson Health uses LLMs to accelerate drug discovery, shaving years off development timelines.

**Education:** Khan Academy's AI tutor, powered by GPT-4, personalizes lessons for 100 million users. Duolingo's Max feature employs LLMs for conversational practice, boosting retention rates by 20%.

**Business and Creativity:** GitHub Copilot, an LLM from Microsoft and OpenAI, writes 46% of code in some repositories, per GitHub stats. Adobe's Firefly integrates LLMs for image generation, while journalists at The Associated Press use them to draft earnings reports 40% faster.

"LLMs are like having a tireless intern who never sleeps," quips Satya Nadella, Microsoft's CEO. In finance, JPMorgan's IndexGPT scans SEC filings to predict market trends with 85% accuracy.

Yet, adoption isn't uniform. Small businesses in developing nations leverage open-source models like Mistral AI's Mixtral, bridging the digital divide. In India, LLM-driven translation apps have made government services accessible in 22 languages, serving 1.4 billion people.

## The Dark Side: Hallucinations, Bias, and Beyond

For all their promise, LLMs have glaring flaws.

**Hallucinations:** These models confidently invent facts. A 2024 Vectara study found GPT-4 hallucinates 27% of the time on factual queries. During the 2024 U.S. election cycle, ChatGPT falsely claimed certain candidates dropped out, spreading via social media.

**Bias:** Trained on internet data rife with prejudice, LLMs perpetuate stereotypes. A Stanford audit revealed Gemini underrepresents women in CEO descriptions by 15%. Anthropic's Claude fares better with "constitutional AI," but no model is bias-proof.

**Job Displacement:** McKinsey predicts 30% of work hours could be automated by 2030, hitting white-collar jobs hardest. Coders, writers, and paralegals report 20-40% productivity gains—but at what cost? The Screenwriters Guild strike in 2023 demanded AI safeguards, fearing "content farms."

Privacy concerns escalate: LLMs ingest user data for fine-tuning. Italy briefly banned ChatGPT in 2023 over GDPR violations. Deepfakes, powered by multimodal LLMs like Sora, threaten elections—Slovakia's 2023 vote saw AI-generated audio nearly sway results.

Existential risks draw heavyweights like Geoffrey Hinton, the "Godfather of AI," who quit Google in 2023 warning of superintelligent LLMs outpacing humans. "If they get smarter than us, we lose control," he told CBS.

## Regulation Races and Global Tensions

Governments scramble to catch up. The EU's AI Act, effective 2025, classifies high-risk LLMs (those serving 45 million+ users) as requiring transparency audits. Violators face 7% of global revenue fines.

In the U.S., Biden's 2023 Executive Order mandates safety testing for models above certain compute thresholds. California proposes watermarking AI-generated content. China mandates government approval for LLMs, prioritizing "socialist values," fueling a U.S.-China tech cold war. Baidu's Ernie Bot rivals GPT, but export controls limit access to advanced chips.

Industry self-regulates: OpenAI's Superalignment team aims for safe superintelligence, though critics call it theater. xAI's Grok emphasizes "maximum truth-seeking" with fewer guardrails.

| Key LLM Milestones | Model | Release Year | Parameters | Breakthrough |
|--------------------|--------|--------------|------------|--------------|
| GPT-3 | OpenAI | 2020 | 175B | Scalable in-context learning |
| PaLM 2 | Google | 2022 | 540B | Multilingual mastery |
| Llama 2 | Meta | 2023 | 70B | Open-source accessibility |
| GPT-4o | OpenAI | 2024 | ~1.76T | Real-time voice/video |
| o1 | OpenAI | 2024 | Undisclosed | Chain-of-thought reasoning |
| Claude 3.5 | Anthropic | 2024 | Undisclosed | Coding supremacy |

*Table: Evolution of Leading LLMs (Sources: Company reports, Epoch AI)*

## Voices from the Frontier

Experts diverge. Optimist Andrej Karpathy, ex-OpenAI, tweets: "LLMs will amplify human intelligence 10x." Pessimist Timnit Gebru warns of "digital colonialism," where Big Tech hoards power.

Yann LeCun of Meta dismisses doomsday scenarios: "LLMs lack real understanding; they're glorified autocomplete." Elon Musk, via xAI, pushes for decentralized AI to democratize benefits.

Users chime in. "It's revolutionized my novel-writing," says author Naomi K. Lewis. But teacher Maria Gonzalez laments: "Students copy-paste AI essays; critical thinking erodes."

## The Road Ahead: Multimodal, Agentic, and Beyond

What's next? Multimodal LLMs like GPT-4V process vision and text, powering robotics. "Agentic" systems—LLMs that plan, execute tasks via APIs—emerge: Devin AI codes full apps autonomously.

AGI whispers grow. OpenAI CEO Sam Altman predicts "superintelligence" by 2027-2030. Compute scales exponentially; by 2025, models could hit 100 trillion parameters.

Sustainability bites: Training GPT-4 emitted 552 tons of CO2, per estimates—equivalent to 120 roundtrip flights NYC-London. Efficient "mixture of experts" architectures mitigate this.

In conclusion, LLMs stand at a pivotal crossroads. They've democratized knowledge, accelerated innovation, and sparked profound societal shifts. Yet, unchecked, they risk amplifying humanity's flaws. As Dr. Li puts it: "AI is a mirror. It reflects our data, our values. We must shape it wisely."

Policymakers, ethicists, and innovators must collaborate. The genie is out; the challenge is harnessing its power without burning the house down.

*Elena Vasquez covers AI for Global Tech Daily. Reach her at elena.vasquez@globaltechdaily.com.*
"#;
}

mod eval {
    use super::ID;
    use crate::feat::bench::prelude::*;

    register_eval!(eval);

    fn eval(_: &[Choice]) -> Score {
        Score::pass()
    }
}
