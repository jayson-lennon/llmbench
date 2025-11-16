use std::{
    collections::{HashMap, VecDeque},
    hash::{Hash, Hasher},
};

use bon::Builder;
use openrouter::completions::{
    Request,
    request::{Message, Stop, Tool, ToolChoice, Usage},
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_64;

use crate::feat::{
    bench::{AllBenchResults, BenchCtx, Benches},
    completion::{RunPayload, worker::RunHash},
    models::{ModelId, Models},
};

const SEED: u64 = 1337;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PromptHash(pub u64);

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize, Builder)]
pub struct PromptRequest {
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

impl PromptRequest {
    /// Return the hash of this request.
    ///
    /// Used to identify this specific request.
    ///
    /// The hash is computed from user messages, system messages, and tool messages. Responses from
    /// LLMs added as part of the request are ignored for the purposes of hashing.
    pub fn prompt_hash(&self) -> PromptHash {
        // Filter out all LLM responses so we get a consistent hash based only on known inputs.
        let messages = self
            .messages
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|msg| {
                matches!(msg, Message::User { .. })
                    || matches!(msg, Message::Tool { .. })
                    || matches!(msg, Message::System { .. })
            })
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

#[derive(Debug)]
pub struct PromptPayloadBatch {
    payloads: Vec<RunPayload>,
}

impl PromptPayloadBatch {
    /// Requests are usually rate-limited per-model or per-provider. This method takes all the
    /// requests and batches them up per-model.
    ///
    /// This enables spawning multiple tasks per model to make concurrent requests.
    pub fn split_by_models(self) -> HashMap<ModelId, VecDeque<RunPayload>> {
        let mut map: HashMap<ModelId, VecDeque<RunPayload>> = HashMap::new();
        for payload in self.payloads {
            let entry = map.entry(payload.ctx.model.clone()).or_default();
            entry.push_back(payload);
        }
        map
    }

    /// Remove already-completed prompt runs
    pub fn filter_old_runs(&mut self, results: AllBenchResults) {
        let mut total_filtered = 0;
        let initial_result_len = self.payloads.len();
        for result in results {
            self.payloads
                .retain(|payload| payload.get_run_hash() != result.hash);
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

    pub fn new(models: Models, benches: &Benches, n_runs: u32) -> PromptPayloadBatch {
        let mut payloads = Vec::new();
        for model in models {
            for bench in benches {
                for run in 0..n_runs {
                    payloads.push(RunPayload {
                        ctx: BenchCtx {
                            run_number: run,
                            model: ModelId(model.clone()),
                            run_hash: RunHash(0),
                        },
                        bench: bench.clone(),
                    });
                }
            }
        }

        Self { payloads }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, RunPayload> {
        self.payloads.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, RunPayload> {
        self.payloads.iter_mut()
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
