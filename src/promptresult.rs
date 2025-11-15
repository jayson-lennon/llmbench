use openrouter::completions::Response;
use serde::{Deserialize, Serialize};

use crate::{
    bench_loader::BenchId,
    promptrequest::{PromptHash, PromptRequest},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    /// Hash generated from the `PromptRequest`. This prevents duplicate requests.
    pub hash: PromptHash,
    /// The name of the bench.
    pub bench: BenchId,
    /// All data sent
    pub request: PromptRequest,
    /// All responses
    pub responses: Vec<Response>,
}
