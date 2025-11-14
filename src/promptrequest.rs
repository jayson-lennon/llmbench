use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use openrouter::completions::{
    Request,
    request::{Content, Message, Stop, Tool, ToolChoice, Usage},
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_64;

use crate::{
    bench_loader::Benches,
    completion::RunPayload,
    models::{ModelId, Models},
    results_dump::ResultsDump,
};

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

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct PromptPayloadBatch {
    payloads: Vec<RunPayload>,
}

impl PromptPayloadBatch {
    /// Requests are usually rate-limited per-model or per-provider. This method takes all the
    /// requests and batches them up per-model.
    ///
    /// This enables spawning multiple tasks per model to make concurrent requests.
    pub fn break_into_models(self) -> HashMap<ModelId, Vec<RunPayload>> {
        let mut map: HashMap<ModelId, Vec<RunPayload>> = HashMap::new();
        for payload in self.payloads {
            let entry = map
                .entry(ModelId(payload.prompt.model.clone()))
                .or_default();
            entry.push(payload);
        }
        map
    }

    /// Remove already-completed prompt runs
    pub fn filter_old_runs(&mut self, results: ResultsDump) {
        let mut total_filtered = 0;
        let initial_result_len = self.payloads.len();
        for result in results {
            self.payloads
                .retain(|payload| payload.prompt.prompt_hash() != result.hash);
        }
        let diff = initial_result_len - self.payloads.len();
        total_filtered += diff;
        tracing::info!(
            requests = initial_result_len,
            filtered = total_filtered,
            pending = self.payloads.len(),
            "filtered benches"
        );
    }

    pub fn new(models: Models, benches: Benches, n_runs: u32) -> PromptPayloadBatch {
        let mut payloads = Vec::new();
        for model in models {
            for bench in &benches {
                for run in 0..n_runs {
                    let prompt = PromptRequest {
                        // +1 here because the run counter starts at 1
                        run_number: run + 1,
                        model: model.clone(),
                        messages: Some({
                            bench
                                .prompts
                                .iter()
                                .map(|prompt| Message::User {
                                    content: Content::Plain(prompt.clone()),
                                    name: None,
                                    cache_control: None,
                                })
                                .collect()
                        }),
                        ..Default::default()
                    };
                    payloads.push(RunPayload {
                        bench: bench.clone(),
                        prompt,
                    });
                }
            }
        }

        Self { payloads }
    }
}

impl IntoIterator for PromptPayloadBatch {
    type Item = RunPayload;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.payloads.into_iter()
    }
}

impl<'a> IntoIterator for &'a PromptPayloadBatch {
    type Item = &'a RunPayload;
    type IntoIter = std::slice::Iter<'a, RunPayload>;

    fn into_iter(self) -> Self::IntoIter {
        self.payloads.iter()
    }
}

impl<'a> IntoIterator for &'a mut PromptPayloadBatch {
    type Item = &'a mut RunPayload;
    type IntoIter = std::slice::IterMut<'a, RunPayload>;

    fn into_iter(self) -> Self::IntoIter {
        self.payloads.iter_mut()
    }
}

impl Extend<RunPayload> for PromptPayloadBatch {
    fn extend<T: IntoIterator<Item = RunPayload>>(&mut self, iter: T) {
        self.payloads.extend(iter);
    }
}

impl FromIterator<RunPayload> for PromptPayloadBatch {
    fn from_iter<T: IntoIterator<Item = RunPayload>>(iter: T) -> Self {
        Self {
            payloads: iter.into_iter().collect(),
        }
    }
}
