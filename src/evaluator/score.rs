use std::collections::HashMap;

use bon::Builder;
use openrouter::completions::response::Choice;
use serde::{Deserialize, Serialize};

use crate::{bench_loader::BenchId, bench_result::BenchResult, models::ModelId};

pub trait GetMessageExt {
    /// Returns a message (if any).
    fn get_message(&self) -> Option<String>;
}

impl GetMessageExt for &Choice {
    fn get_message(&self) -> Option<String> {
        match self {
            Choice::NonStreaming(choice) => choice.message.content.clone(),
            _ => unimplemented!("only non-streaming responses are supported"),
        }
    }
}

/// A scored response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredResponse {
    pub response: BenchResult,
    pub score: Score,
}

/// The result of an evaluator.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Score {
    /// When true, the model passed the bench
    pub pass: bool,

    /// Total cost incurred to run this bench.
    ///
    /// This will be filled in automatically by the evaluator harness.
    #[builder(default)]
    pub cost: f64,

    /// Total tokens used to output the response.
    ///
    /// This will be filled in automatically by the evaluator harness.
    #[builder(default)]
    pub completion_tokens: u32,
}

impl Score {
    /// Return a default failing score.
    pub fn fail() -> Self {
        Score::builder().pass(false).build()
    }

    /// Return a default passing score.
    pub fn pass() -> Self {
        Score::builder().pass(true).build()
    }
}

/// Key used to calculate total number of runs for a given bench+model combination
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BenchModelKey {
    pub bench_id: BenchId,
    pub model_id: ModelId,
}

/// All the scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scores {
    scores: HashMap<BenchModelKey, Vec<ScoredResponse>>,
}

impl Scores {
    pub fn get(&self, k: &BenchModelKey) -> Option<&Vec<ScoredResponse>> {
        self.scores.get(k)
    }

    pub fn values(
        &self,
    ) -> std::collections::hash_map::Values<'_, BenchModelKey, Vec<ScoredResponse>> {
        self.scores.values()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, BenchModelKey, Vec<ScoredResponse>> {
        self.scores.iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> std::collections::hash_map::IterMut<'_, BenchModelKey, Vec<ScoredResponse>> {
        self.scores.iter_mut()
    }
}

impl IntoIterator for Scores {
    type Item = (BenchModelKey, Vec<ScoredResponse>);
    type IntoIter = std::collections::hash_map::IntoIter<BenchModelKey, Vec<ScoredResponse>>;

    fn into_iter(self) -> Self::IntoIter {
        self.scores.into_iter()
    }
}

impl<'a> IntoIterator for &'a Scores {
    type Item = (&'a BenchModelKey, &'a Vec<ScoredResponse>);
    type IntoIter = std::collections::hash_map::Iter<'a, BenchModelKey, Vec<ScoredResponse>>;

    fn into_iter(self) -> Self::IntoIter {
        self.scores.iter()
    }
}

impl<'a> IntoIterator for &'a mut Scores {
    type Item = (&'a BenchModelKey, &'a mut Vec<ScoredResponse>);
    type IntoIter = std::collections::hash_map::IterMut<'a, BenchModelKey, Vec<ScoredResponse>>;

    fn into_iter(self) -> Self::IntoIter {
        self.scores.iter_mut()
    }
}

impl FromIterator<ScoredResponse> for Scores {
    fn from_iter<T: IntoIterator<Item = ScoredResponse>>(iter: T) -> Self {
        let scores: Vec<ScoredResponse> = iter.into_iter().collect();

        let mut aggregated: HashMap<BenchModelKey, Vec<ScoredResponse>> = HashMap::new();

        for scored_response in scores {
            let key = BenchModelKey {
                bench_id: scored_response.response.bench.clone(),
                model_id: ModelId(scored_response.response.request.model.clone()),
            };

            aggregated.entry(key).or_default().push(scored_response);
        }
        Scores { scores: aggregated }
    }
}
