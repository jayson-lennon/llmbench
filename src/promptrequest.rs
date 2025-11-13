use std::hash::{Hash, Hasher};

use openrouter::completions::{
    Request,
    request::{Message, Stop, Tool, ToolChoice, Usage},
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_64;

const SEED: u64 = 1337;

#[derive(Debug, thiserror::Error)]
#[error("prompt request error")]
pub struct PromptRequestError;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PromptHash(pub u64);

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct PromptRequest {
    pub run_number: u32,
    pub model: String,

    pub messages: Option<Vec<Message>>,
    pub prompt: Option<String>,
    pub stop: Option<Stop>,
    pub max_tokens: Option<u64>,
    pub temperature: Option<OrderedFloat<f64>>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub seed: Option<u64>,
    pub top_p: Option<OrderedFloat<f64>>,
    pub top_k: Option<u64>,
    pub frequency_penalty: Option<OrderedFloat<f64>>,
    pub presence_penalty: Option<OrderedFloat<f64>>,
    pub repetition_penalty: Option<OrderedFloat<f64>>,
    pub min_p: Option<OrderedFloat<f64>>,
    pub top_a: Option<OrderedFloat<f64>>,
}

impl Default for PromptRequest {
    fn default() -> Self {
        PromptRequest {
            run_number: 1,
            model: String::new(),
            messages: None,
            prompt: None,
            stop: None,
            max_tokens: None,
            temperature: None,
            tools: None,
            tool_choice: None,
            seed: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            min_p: None,
            top_a: None,
        }
    }
}

impl PromptRequest {
    /// Return the hash of this request.
    ///
    /// Used to identify this specific request.
    pub fn prompt_hash(&self) -> PromptHash {
        // Filter out all LLM responses so we get a consistent hash based only on known inputs.
        let messages = self
            .messages
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|msg| matches!(msg, Message::User { .. }))
            .collect::<Vec<_>>();

        let mut request = self.clone();
        request.messages = Some(messages);

        let mut hasher = XxHash3_64::with_seed(SEED);
        request.hash(&mut hasher);
        let hash = hasher.finish();
        PromptHash(hash)
    }

    /// Creates a new openrouter request from this prompt request.
    pub fn make_openrouter_request(&self) -> Request {
        Request {
            model: Some(self.model.clone()),
            messages: self.messages.clone(),
            prompt: self.prompt.clone(),
            stop: self.stop.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature.map(|f| f.0),
            tools: self.tools.clone(),
            tool_choice: self.tool_choice.clone(),
            seed: self.seed,
            top_p: self.top_p.map(|f| f.0),
            top_k: self.top_k,
            frequency_penalty: self.frequency_penalty.map(|f| f.0),
            presence_penalty: self.presence_penalty.map(|f| f.0),
            repetition_penalty: self.repetition_penalty.map(|f| f.0),
            min_p: self.min_p.map(|f| f.0),
            top_a: self.top_a.map(|f| f.0),
            usage: Some(Usage { include: true }),
            ..Default::default()
        }
    }
}
