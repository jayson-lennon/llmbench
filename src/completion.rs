use error_stack::Report;
use openrouter::OpenRouter;
use serde::{Deserialize, Serialize};

use crate::{
    bench_loader::Bench,
    models::ModelId,
    promptrequest::PromptRequest,
    promptresult::PromptResponse,
    result_writer::{ResultSender, ResultWriterCmd},
};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct RunPayload {
    pub bench: Bench,
    pub prompt: PromptRequest,
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub api_key: String,
    pub results_tx: ResultSender,
}

#[derive(Debug, thiserror::Error)]
#[error("a RunCompletion occurred")]
pub struct RunCompletion;

#[tracing::instrument(skip(config, payloads), err)]
async fn start_impl(
    config: RunConfig,
    model: ModelId,
    payloads: Vec<RunPayload>,
) -> Result<(), Report<RunCompletion>> {
    let openrouter = OpenRouter::new(config.api_key);
    for payload in payloads {
        let hash = payload.prompt.prompt_hash();
        tracing::info!(model=%model, bench=%payload.bench.id, "sending completion request");
        match openrouter
            .chat_completion(payload.prompt.make_openrouter_request())
            .await
        {
            Ok(response) => {
                tracing::debug!(model=%model, bench=%payload.bench.id, "got response");
                let result = PromptResponse {
                    hash,
                    bench: payload.bench.id,
                    request: payload.prompt,
                    responses: vec![response],
                };
                config
                    .results_tx
                    .send(ResultWriterCmd::SaveResult(result))
                    .unwrap();
            }
            Err(e) => {
                tracing::error!(err=?e, "failed to get chat completion");
            }
        }
    }
    Ok(())
}

pub async fn start(config: RunConfig, model: ModelId, payloads: Vec<RunPayload>) {
    if let Err(e) = start_impl(config, model.clone(), payloads).await {
        tracing::error!(model=%model, err=?e, "an error occurred while processing a request")
    }
}
