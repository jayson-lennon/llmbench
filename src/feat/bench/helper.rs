use std::sync::OnceLock;

use openrouter::completions::{
    Response,
    request::{Content, Message},
    response::Choice,
};
use regex::Regex;

/// Register a new benchmark into the distributed slice
macro_rules! register_bench {
    ($bench:ident) => {
        #[distributed_slice(BENCHMARKS)]
        static BENCHMARK: BenchInit = init;

        fn init() -> Bench {
            Bench::new(BenchId(ID.to_string()), $bench)
        }
    };
}
pub(crate) use register_bench;

/// Register a new evaluator into the distributed slice
macro_rules! register_eval {
    ($eval:ident) => {
        #[distributed_slice(EVALUATORS)]
        static EVALUATOR: EvaluatorInit = init;

        fn init() -> Evaluator {
            Evaluator {
                bench: BenchId(ID.to_string()),
                eval: $eval,
            }
        }
    };
}
pub(crate) use register_eval;

/// Create a new user message.
pub(in crate::feat::bench) fn user_message<M>(msg: M) -> Message
where
    M: Into<String>,
{
    let msg = msg.into();
    Message::User {
        content: Content::Plain(msg),
        name: None,
        cache_control: None,
    }
}

/// Create a new assistant message.
pub(in crate::feat::bench) fn assistant_message<M>(msg: M) -> Message
where
    M: Into<String>,
{
    let msg = msg.into();
    Message::Assistant {
        content: Some(Content::Plain(msg)),
        name: None,
        tool_calls: None,
    }
}

pub(in crate::feat::bench) trait ResponseExt {
    fn get_assistant_message(&self) -> Option<String>;
}

impl ResponseExt for Response {
    fn get_assistant_message(&self) -> Option<String> {
        if let Some(choice) = self.choices.first() {
            match choice {
                Choice::NonStreaming(choice) => choice.message.content.clone(),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn chat_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"<\|.*?\|>").unwrap())
}

pub(in crate::feat::bench) trait StringBenchExt {
    fn alphanumeric_only(&self) -> String;
    fn lowercase(&self) -> String;
    fn remove_chat_tags(&self) -> String;
}

impl StringBenchExt for String {
    /// Includes:
    /// - characters
    /// - numbers
    /// - whitespace
    fn alphanumeric_only(&self) -> String {
        self.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    fn lowercase(&self) -> String {
        self.to_lowercase()
    }

    /// Removes `<|tag|>` from the response.
    fn remove_chat_tags(&self) -> String {
        let re = chat_tag_re();
        re.replace_all(self, "").to_string()
    }
}

impl StringBenchExt for &'static str {
    /// Includes:
    /// - characters
    /// - numbers
    /// - whitespace
    fn alphanumeric_only(&self) -> String {
        self.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    fn lowercase(&self) -> String {
        self.to_lowercase()
    }

    fn remove_chat_tags(&self) -> String {
        let re = chat_tag_re();
        re.replace_all(self, "").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_chat_tags() {
        let input = "<|begin_of_box|>Pick a game engine<|end_of_box|>";
        let input = input.remove_chat_tags();
        assert_eq!(input, "Pick a game engine");
    }
}
