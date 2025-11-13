use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{error::Error, providers::Provider};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub id: String,
    pub provider: Provider,
    pub choices: Vec<Choice>,
    #[serde(with = "time::serde::timestamp")]
    pub created: OffsetDateTime,
    pub model: String,
    pub object: Object,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<ResponseUsage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub enum Object {
    #[serde(rename = "chat.completion")]
    Completion,
    #[serde(rename = "chat.completion.chunk")]
    Chunk,
}

/// Usage statistics.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct ResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cost: Option<u32>,
    pub prompt_tokens_details: Option<PromptTokenDetails>,
    pub completion_tokens_details: Option<CompletionTokenDetails>,
    pub total_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PromptTokenDetails {
    pub cached_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletionTokenDetails {
    pub reasoning_tokens: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
#[serde(untagged)]
pub enum Choice {
    NonChat(NonChatChoice),
    NonStreaming(NonStreamingChoice),
    Streaming(StreamingChoice),
}

/// A non‑chat (plain text) choice.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct NonChatChoice {
    pub logprobs: Option<LogProbs>,
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    pub text: String,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub error: Option<Error>,
}

/// A non‑streaming chat choice, which carries a complete message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct NonStreamingChoice {
    // Have not yet figured out the type.
    #[serde(default)]
    pub logprobs: Option<LogProbs>,
    // Could probably be an enum.
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    pub index: usize,
    pub message: Message,
    #[serde(default)]
    pub error: Option<Error>,
}

/// A streaming chat choice, which carries a delta update.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct StreamingChoice {
    #[serde(default)]
    pub finish_reason: Option<String>,
    pub delta: Delta,
    #[serde(default)]
    pub error: Option<Error>,
}

// Currently not implemented. Likely equal to this one:
// https://platform.openai.com/docs/api-reference/chat/object
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct LogProbs {
    pub content: Vec<()>,
    pub refusal: Vec<()>,
}

/// A full message for non‑streaming choices.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Message {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A delta update for streaming choices.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Delta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Hash, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct ToolCall {
    pub id: String,
    pub index: usize,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionCall,
}

#[derive(Hash, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use pretty_assertions::assert_eq;
    use time::{Date, Month, OffsetDateTime, Time};

    use super::*;

    #[test]
    fn chat_completion() {
        let file = read_to_string("./responses/chat-completion.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Response {
            id: "gen-1738698054-yI7fNDyuNM2VdvsYZ6Po".to_string(),
            provider: Provider::GoogleAIStudio,
            choices: vec![Choice::NonStreaming(NonStreamingChoice {
                logprobs: None,
                index: 0,
                finish_reason: None,
                native_finish_reason: None,
                message: Message {
                    content: Some("I am a large language model, trained by Google.".to_owned()),
                    role: "assistant".to_string(),
                    refusal: None,
                    reasoning: None,
                    tool_calls: None,
                },
                error: None,
            })],
            created: OffsetDateTime::new_utc(
                Date::from_calendar_date(2025, Month::February, 4).unwrap(),
                Time::from_hms(19, 40, 54).unwrap(),
            ),
            model: "google/gemini-2.0-flash-thinking-exp".to_string(),
            object: Object::Completion,
            system_fingerprint: None,
            usage: Some(ResponseUsage {
                prompt_tokens: 18,
                completion_tokens: 11,
                cost: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                total_tokens: 29,
            }),
        };

        assert_eq!(expected, response);
    }

    #[test]
    fn completion() {
        let file = read_to_string("./responses/completion.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Response {
            id: "gen-1738697078-nbXebb9pscDuajFRYYob".to_string(),
            provider: Provider::Targon,
            choices: vec![Choice::NonChat(NonChatChoice {
                finish_reason: None,
                logprobs: Some(LogProbs {
                    content: vec![],
                    refusal: vec![],
                }),
                text: "This is a shortened text output".to_string(),
                reasoning: None,
                error: None,
                native_finish_reason: None,
            })],
            created: OffsetDateTime::new_utc(
                Date::from_calendar_date(2025, Month::February, 4).unwrap(),
                Time::from_hms(19, 24, 38).unwrap(),
            ),
            model: "deepseek/deepseek-r1-distill-llama-70b".to_string(),
            object: Object::Completion,
            system_fingerprint: None,
            usage: Some(ResponseUsage {
                prompt_tokens: 4,
                completion_tokens: 589,
                cost: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                total_tokens: 593,
            }),
        };

        assert_eq!(expected, response);
    }

    #[test]
    fn tool_call() {
        let file = read_to_string("./responses/completion/chat/2025-03-07-tool-call.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn reasoning() {
        let file = read_to_string("./responses/completion/chat/2025-03-07-reasoning.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn simple() {
        let file = read_to_string("./responses/completion/normal/2025-03-07.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn unknown() {
        let file =
            read_to_string("./responses/completion/chat/2025-03-30-tools-and-reasoning.json")
                .unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }
}
