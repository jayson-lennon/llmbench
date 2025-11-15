pub mod container;
pub mod loader;

use std::str::FromStr;

pub use container::{AllBenchResults, Bench, Benches};

use derive_more::Display;
use error_stack::{Report, ResultExt};
use openrouter::completions::Response;
use serde::{Deserialize, Serialize};

use crate::feat::completion::{PromptHash, PromptRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    /// Hash generated from the `PromptRequest`. This prevents duplicate requests.
    pub hash: PromptHash,
    /// The name of the bench.
    pub bench: BenchId,
    /// All data sent
    pub request: PromptRequest,
    /// All responses
    pub responses: Vec<Response>,
}

#[derive(Debug, thiserror::Error)]
#[error("a BenchError occurred")]
pub struct BenchError;

#[derive(Display, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[display("{_0}")]
pub struct BenchId(pub String);

impl BenchId {
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromStr for BenchId {
    type Err = Report<BenchError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((category, name)) = s.split_once('/') {
            if category.is_empty() {
                return Err(Report::from(BenchError)).attach("missing category from bench id");
            }
            if name.is_empty() {
                return Err(Report::from(BenchError)).attach("missing name from bench id");
            }
            Ok(Self(s.to_string()))
        } else {
            Err(Report::from(BenchError)).attach("invalid id format (must be category/benchname)")
        }
    }
}
