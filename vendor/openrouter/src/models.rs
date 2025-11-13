//! <https://openrouter.ai/api/v1/models>

use std::{fmt::Display, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_this_or_that::{as_f64, as_opt_f64};
use time::OffsetDateTime;

pub mod count;
pub mod endpoints;

/// OpenRouter puts the returned data into a `data` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub data: Vec<Model>,
}

impl Deref for Response {
    type Target = [Model];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Model {
    pub id: String,
    pub hugging_face_id: Option<String>,
    pub name: String,
    #[serde(with = "time::serde::timestamp")]
    pub created: OffsetDateTime,
    pub description: Option<String>,
    pub pricing: Pricing,
    pub context_length: usize,
    pub architecture: Architecture,
    pub top_provider: TopProvider,
    pub per_request_limits: Option<PerRequestLimits>,
    pub supported_parameters: Vec<Parameter>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Pricing {
    #[serde(deserialize_with = "as_f64")]
    pub prompt: f64,
    #[serde(deserialize_with = "as_f64")]
    pub completion: f64,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub image: Option<f64>,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub request: Option<f64>,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub input_cache_read: Option<f64>,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub input_cache_write: Option<f64>,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub web_search: Option<f64>,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub internal_reasoning: Option<f64>,
    #[serde(deserialize_with = "as_opt_f64", default)]
    pub discount: Option<f64>,
}

impl Pricing {
    pub fn free(&self) -> bool {
        self.prompt + self.completion + self.image.unwrap_or(0.0) + self.request.unwrap_or(0.0)
            == 0.0
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Architecture {
    #[deprecated(
        note = "OpenRouter removed this option, use input_modalities and output_modalities instead"
    )]
    pub modality: Option<String>,
    pub input_modalities: Vec<Modality>,
    pub output_modalities: Vec<Modality>,
    pub tokenizer: Tokenizer,
    pub instruct_type: Option<InstructType>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Modality {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "file")]
    File,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Tokenizer {
    #[serde(rename = "Router")]
    Router,
    #[serde(rename = "Media")]
    Media,
    #[serde(rename = "Other")]
    Other,
    #[serde(rename = "GPT")]
    GPT,
    #[serde(rename = "Claude")]
    Claude,
    #[serde(rename = "Gemini")]
    Gemini,
    #[serde(rename = "Grok")]
    Grok,
    #[serde(rename = "Cohere")]
    Cohere,
    #[serde(rename = "Nova")]
    Nova,
    #[serde(rename = "Qwen")]
    Qwen,
    #[serde(rename = "Qwen3")]
    Qwen3,
    #[serde(rename = "Yi")]
    Yi,
    #[serde(rename = "DeepSeek")]
    DeepSeek,
    #[serde(rename = "Mistral")]
    Mistral,
    #[serde(rename = "Llama2")]
    Llama2,
    #[serde(rename = "Llama3")]
    Llama3,
    #[serde(rename = "Llama4")]
    Llama4,
    #[serde(rename = "PaLM")]
    PaLM,
    #[serde(rename = "RWKV")]
    RWKV,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum InstructType {
    #[serde(rename = "None", alias = "none")]
    None,
    #[serde(rename = "Airoboros", alias = "airoboros")]
    Airoboros,
    #[serde(rename = "Alpaca", alias = "alpaca")]
    Alpaca,
    #[serde(rename = "AlpacaModif")]
    AlpacaModif,
    #[serde(rename = "ChatML", alias = "chatml")]
    ChatML,
    #[serde(rename = "Claude")]
    Claude,
    #[serde(rename = "CodeLlama", alias = "code-llama")]
    CodeLlama,
    #[serde(rename = "deepseek-r1")]
    DeepSeekR1,
    #[serde(rename = "Gemma", alias = "gemma")]
    Gemma,
    #[serde(rename = "Llama2", alias = "llama2")]
    Llama2,
    #[serde(rename = "Llama3", alias = "llama3")]
    Llama3,
    #[serde(rename = "Mistral", alias = "mistral")]
    Mistral,
    #[serde(rename = "Nemotron")]
    Nemotron,
    #[serde(rename = "Neural")]
    Neural,
    #[serde(rename = "OpenChat", alias = "openchat")]
    OpenChat,
    #[serde(rename = "Phi3", alias = "phi3")]
    Phi3,
    #[serde(rename = "qwen3")]
    Qwen3,
    #[serde(rename = "qwq")]
    QwQ,
    #[serde(rename = "RWKV")]
    RWKV,
    #[serde(rename = "Vicuna", alias = "vicuna")]
    Vicuna,
    #[serde(rename = "Zephyr", alias = "zephyr")]
    Zephyr,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct TopProvider {
    pub context_length: Option<usize>,
    pub max_completion_tokens: Option<usize>,
    pub is_moderated: bool,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct PerRequestLimits {
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parameter {
    FrequencyPenalty,
    IncludeReasoning,
    LogitBias,
    LogProbs,
    MaxTokens,
    MinP,
    PresencePenalty,
    Reasoning,
    RepetitionPenalty,
    ResponseFormat,
    Seed,
    Stop,
    StructuredOutputs,
    Temperature,
    ToolChoice,
    Tools,
    TopA,
    TopK,
    TopLogProbs,
    TopP,
    WebSearchOptions,
    Unknown(String),
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Parameter::FrequencyPenalty => "frequency_penalty",
            Parameter::IncludeReasoning => "include_reasoning",
            Parameter::LogitBias => "logit_bias",
            Parameter::LogProbs => "logprobs",
            Parameter::MaxTokens => "max_tokens",
            Parameter::MinP => "min_p",
            Parameter::PresencePenalty => "presence_penalty",
            Parameter::Reasoning => "reasoning",
            Parameter::RepetitionPenalty => "repetition_penalty",
            Parameter::ResponseFormat => "response_format",
            Parameter::Seed => "seed",
            Parameter::Stop => "stop",
            Parameter::StructuredOutputs => "structured_outputs",
            Parameter::Temperature => "temperature",
            Parameter::ToolChoice => "tool_choice",
            Parameter::Tools => "tools",
            Parameter::TopA => "top_a",
            Parameter::TopK => "top_k",
            Parameter::TopLogProbs => "top_logprobs",
            Parameter::TopP => "top_p",
            Parameter::WebSearchOptions => "web_search_options",
            Parameter::Unknown(u) => u,
        };

        f.serialize_str(str)
    }
}

impl Serialize for Parameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self)
    }
}

impl<'de> Deserialize<'de> for Parameter {
    fn deserialize<D>(deserializer: D) -> Result<Parameter, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // While we could just match the lowercase version, not all of them
        // can be found in the API. This may change in the future.
        // The below is therefore a mix of what the OpenAPI spec gives us and what the API returned.
        // A support ticket regarding this has been opened with OpenRouter.
        let parameter = match s.as_str() {
            "frequency_penalty" => Parameter::FrequencyPenalty,
            "include_reasoning" => Parameter::IncludeReasoning,
            "logit_bias" => Parameter::LogitBias,
            "logprobs" => Parameter::LogProbs,
            "max_tokens" => Parameter::MaxTokens,
            "min_p" => Parameter::MinP,
            "presence_penalty" => Parameter::PresencePenalty,
            "reasoning" => Parameter::Reasoning,
            "repetition_penalty" => Parameter::RepetitionPenalty,
            "response_format" => Parameter::ResponseFormat,
            "seed" => Parameter::Seed,
            "stop" => Parameter::Stop,
            "structured_outputs" => Parameter::StructuredOutputs,
            "temperature" => Parameter::Temperature,
            "tool_choice" => Parameter::ToolChoice,
            "tools" => Parameter::Tools,
            "top_a" => Parameter::TopA,
            "top_k" => Parameter::TopK,
            "top_logprobs" => Parameter::TopLogProbs,
            "top_p" => Parameter::TopP,
            "web_search_options" => Parameter::WebSearchOptions,
            other => Parameter::Unknown(other.to_owned()),
        };

        Ok(parameter)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::Response;

    #[test]
    fn models() {
        let file = read_to_string("./responses/models/2025-05-28.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        assert_eq!(316, response.data.len());
        assert_eq!(
            "2025-05-25 15:53:33.0 +00:00:00",
            response.data[0].created.to_string()
        )
    }
}
