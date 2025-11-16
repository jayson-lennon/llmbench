use std::hash::{Hash, Hasher};
use std::{collections::VecDeque, sync::Arc};

use error_stack::{Report, ResultExt};
use openrouter::OpenRouter;
use openrouter::completions::Response;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_64;

use crate::feat::bench::BenchId;
use crate::feat::completion::PromptRequest;
use crate::feat::{
    bench::{Bench, BenchCtx},
    models::ModelId,
    persistence::{ResultSender, ResultWriterCmd},
};

/// Unique hash assigned to a model+run+bench combination.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RunHash(pub u64);

#[derive(Debug)]
pub struct RunPayload {
    pub ctx: BenchCtx,
    pub bench: Bench,
}

impl RunPayload {
    /// Returns the run hash for this particular model+run+bench combo.
    ///
    /// Note that this is NOT a hash of the [`RunPayload`] struct! It's used only to prevent
    /// duplicate requests from being sent out.
    pub fn get_run_hash(&self) -> RunHash {
        const SEED: u64 = 1337;

        let data = (self.ctx.clone(), self.bench.id.clone());

        let mut hasher = XxHash3_64::with_seed(SEED);
        data.hash(&mut hasher);
        RunHash(hasher.finish())
    }
}

/// Configuration required to start a new completion run.
#[derive(Debug, Clone)]
pub struct RunConfig {
    pub api_key: String,
    pub results_tx: ResultSender,
}

#[derive(Debug, thiserror::Error)]
#[error("a RunCompletion occurred")]
pub struct CompletionError;

/// Implementation to start and execute a chat session.
#[tracing::instrument(skip(config, payloads), err)]
async fn run_impl(
    config: RunConfig,
    model: ModelId,
    mut payloads: VecDeque<RunPayload>,
) -> Result<(), Report<CompletionError>> {
    let openrouter = Arc::new(OpenRouter::new(config.api_key));

    while let Some(mut payload) = payloads.pop_front() {
        payload.ctx.run_hash = payload.get_run_hash();
        let bench = payload
            .bench
            .create_callback(Arc::clone(&openrouter), payload.ctx);
        match bench.await {
            Ok(result) => {
                config
                    .results_tx
                    .send(ResultWriterCmd::SaveResult(result))
                    .unwrap();
            }
            Err(e) => {
                tracing::error!(err=?e, "error");
                return Err(e);
            }
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CompletionErrorWrapper {
    #[error("openrouter error {_0}")]
    OpenRouter(#[from] openrouter::error::Error),
}

/// Run a completion request.
pub async fn complete(
    api: &OpenRouter,
    request: PromptRequest,
    model: &ModelId,
    bench: &BenchId,
) -> Result<Response, Report<CompletionError>> {
    tracing::info!(model=%model, bench=%bench, "sending completion request");

    // Add 1 minute timeout
    let timeout_duration = std::time::Duration::from_secs(60);

    match tokio::time::timeout(
        timeout_duration,
        api.chat_completion(request.make_openrouter_request()),
    )
    .await
    {
        Ok(Ok(response)) => {
            tracing::debug!(model=%model, bench=%bench, "got response");
            Ok(response)
        }
        Ok(Err(e)) => {
            tracing::error!(err=?e, "failed to get chat completion");
            match e {
                openrouter::Error::OpenRouter(e) => {
                    tracing::error!(err=?e, "an openrouter error occurred");
                    Err(CompletionErrorWrapper::OpenRouter(e))
                        .change_context(CompletionError)
                        .attach("failed to get chat completion")
                }
                e => {
                    tracing::error!(err=?e, "an misc worker error occurred");
                    Err(e)
                        .change_context(CompletionError)
                        .attach("failed to get chat completion")
                }
            }
        }
        Err(_) => {
            tracing::error!("chat completion request timed out after 1 minute");
            Err(Report::new(CompletionError)).attach("timed out")
        }
    }
}
//
pub async fn run(config: RunConfig, model: ModelId, payloads: VecDeque<RunPayload>) {
    if let Err(e) = run_impl(config, model.clone(), payloads).await {
        tracing::error!(model=%model, err=?e, "an error occurred while processing a request");
    }
}
