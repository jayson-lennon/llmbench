use bon::Builder;
use openrouter::completions::response::Choice;
use serde::{Deserialize, Serialize};

use crate::promptresult::PromptResponse;

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
    pub response: PromptResponse,
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

/// All the scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scores {
    inner: Vec<ScoredResponse>,
}

impl IntoIterator for Scores {
    type Item = ScoredResponse;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Scores {
    type Item = &'a ScoredResponse;
    type IntoIter = std::slice::Iter<'a, ScoredResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Scores {
    type Item = &'a mut ScoredResponse;
    type IntoIter = std::slice::IterMut<'a, ScoredResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl FromIterator<ScoredResponse> for Scores {
    fn from_iter<T: IntoIterator<Item = ScoredResponse>>(iter: T) -> Self {
        Scores {
            inner: iter.into_iter().collect(),
        }
    }
}

impl Extend<ScoredResponse> for Scores {
    fn extend<T: IntoIterator<Item = ScoredResponse>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}
