pub mod composable;

use crate::feat::completion::PromptRequest;
use crate::feat::completion::worker::{CompletionError, RunHash};
use crate::feat::model::ModelId;
use derive_more::Debug;
use derive_more::Display;
use error_stack::{Report, ResultExt};
use futures::future::{BoxFuture, FutureExt};
use openrouter::OpenRouter;
use openrouter::completions::{request::Content, response::Choice, Response};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use std::{io::ErrorKind, path::Path};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

// ── Save-to-result traits ───────────────────────────────────────────────────

pub trait BenchResultRequestExt {
    #[must_use]
    fn save_to(self, result: &mut BenchResult) -> Self;
}

impl BenchResultRequestExt for PromptRequest {
    fn save_to(self, result: &mut BenchResult) -> Self {
        result.push_request(self.clone());
        self
    }
}

pub trait BenchResultResponseExt {
    #[must_use]
    fn save_to(self, result: &mut BenchResult) -> Self;
}

impl BenchResultResponseExt for Response {
    fn save_to(self, result: &mut BenchResult) -> Self {
        result.push_response(self.clone());
        self
    }
}

// ── Bench result ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    /// Unique hash generated from model+run number+bench
    pub hash: RunHash,
    /// The name of the bench.
    pub bench: BenchId,
    /// The model used.
    pub model: ModelId,
    /// All messages sent
    pub requests: Vec<PromptRequest>,
    /// All responses
    pub responses: Vec<Response>,
}

impl BenchResult {
    pub fn push_request(&mut self, request: PromptRequest) {
        self.requests.push(request);
    }

    pub fn push_response(&mut self, response: Response) {
        self.responses.push(response);
    }
}

// ── Context ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BenchCtx {
    pub run_number: u32,
    pub model: ModelId,
    pub run_hash: RunHash,
}

// ── Bench ID ────────────────────────────────────────────────────────────────

#[derive(Display, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[display("{_0}")]
pub struct BenchId(pub String);

impl std::str::FromStr for BenchId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl BenchId {
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

// ── All bench results (persistence) ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AllBenchResults {
    pub inner: Vec<BenchResult>,
}

#[derive(Debug, thiserror::Error)]
#[error("an AllBenchResultsError error occurred")]
pub struct AllBenchResultsError;

impl AllBenchResults {
    pub async fn load<P>(path: P) -> Result<Self, Report<AllBenchResultsError>>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new()
            .create(false)
            .read(true)
            .open(&path)
            .await;
        let mut file = match file {
            Ok(file) => file,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => return Ok(Self { inner: vec![] }),
                _ => {
                    return Err(e).change_context(AllBenchResultsError).attach_with(|| {
                        format!(
                            "failed to open results file at '{}'",
                            path.as_ref().display()
                        )
                    });
                }
            },
        };

        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .await
            .change_context(AllBenchResultsError)
            .attach_with(|| {
                format!(
                    "failed to read results file at '{}'",
                    path.as_ref().display()
                )
            })?;

        let mut results = Vec::new();
        for result in buf.lines() {
            let result: BenchResult = serde_json::from_str(result)
                .change_context(AllBenchResultsError)
                .attach("deserialization eror")?;
            results.push(result);
        }
        Ok(Self { inner: results })
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BenchResult> {
        self.inner.iter()
    }
}

impl IntoIterator for AllBenchResults {
    type Item = BenchResult;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a AllBenchResults {
    type Item = &'a BenchResult;
    type IntoIter = std::slice::Iter<'a, BenchResult>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl Extend<BenchResult> for AllBenchResults {
    fn extend<T: IntoIterator<Item = BenchResult>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}

impl FromIterator<BenchResult> for AllBenchResults {
    fn from_iter<T: IntoIterator<Item = BenchResult>>(iter: T) -> Self {
        Self {
            inner: Vec::from_iter(iter),
        }
    }
}

// ── Bench factory ───────────────────────────────────────────────────────────

type BenchCallback = BoxFuture<'static, Result<BenchResult, Report<CompletionError>>>;
type BenchFactory = Arc<dyn Fn(Arc<OpenRouter>, BenchCtx) -> BenchCallback + Send + Sync + 'static>;

/// Stores the function pointer/closure that can generate the BoxFuture.
#[derive(Debug, Clone)]
pub struct Bench {
    pub id: BenchId,
    #[debug(skip)]
    factory: BenchFactory,
}

impl Bench {
    /// Creates a new job by boxing the provided factory closure.
    pub fn new<F, Fut>(id: BenchId, f: F) -> Self
    where
        F: Fn(Arc<OpenRouter>, BenchCtx) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<BenchResult, Report<CompletionError>>> + Send + 'static,
    {
        Bench {
            id,
            factory: Arc::new(move |api, ctx| f(api, ctx).boxed()),
        }
    }

    pub fn create_callback(&self, api: Arc<OpenRouter>, ctx: BenchCtx) -> BenchCallback {
        (self.factory)(api, ctx)
    }
}

// ── Utility traits/functions ────────────────────────────────────────────────

/// Create a new user message.
pub fn user_message<M>(msg: M) -> openrouter::completions::request::Message
where
    M: Into<String>,
{
    let msg = msg.into();
    openrouter::completions::request::Message::User {
        content: Content::Plain(msg),
        name: None,
        cache_control: None,
    }
}

fn chat_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"<\|.*?\|>").unwrap())
}

pub trait StringBenchExt {
    fn alphanumeric_only(&self) -> String;
    fn lowercase(&self) -> String;
    fn remove_chat_tags(&self) -> String;
}

impl StringBenchExt for String {
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

pub trait GetMessageExt {
    fn get_message(&self) -> Option<String>;
}

impl GetMessageExt for &Choice {
    fn get_message(&self) -> Option<String> {
        match self {
            Choice::NonStreaming(choice) => choice.message.content.clone(),
            _ => None,
        }
    }
}

// ── Benches collection ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Benches {
    pub benches: Vec<Bench>,
}

impl Benches {
    pub fn iter(&self) -> std::slice::Iter<'_, Bench> {
        self.benches.iter()
    }
}

impl IntoIterator for Benches {
    type Item = Bench;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.benches.into_iter()
    }
}

impl<'a> IntoIterator for &'a Benches {
    type Item = &'a Bench;
    type IntoIter = std::slice::Iter<'a, Bench>;

    fn into_iter(self) -> Self::IntoIter {
        self.benches.iter()
    }
}

impl FromIterator<Bench> for Benches {
    fn from_iter<I: IntoIterator<Item = Bench>>(iter: I) -> Self {
        Benches {
            benches: iter.into_iter().collect(),
        }
    }
}

impl Extend<Bench> for Benches {
    fn extend<T: IntoIterator<Item = Bench>>(&mut self, iter: T) {
        self.benches.extend(iter);
    }
}
