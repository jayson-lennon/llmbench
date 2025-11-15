use std::collections::VecDeque;

use error_stack::{Report, ResultExt};
use openrouter::{
    OpenRouter,
    completions::{
        Response,
        request::{Content, Message},
        response::Choice,
    },
};
use serde::{Deserialize, Serialize};

use crate::feat::{
    bench::{Bench, BenchId, BenchResult},
    completion::PromptRequest,
    models::ModelId,
    persistence::{ResultSender, ResultWriterCmd},
};

/// Everything needed to perform a benchmark.
///
/// A payload represents a single benchmark for a single model.
///
/// Payloads contain:
/// - the bench info (name, prompts, etc)
/// - the request (all config for sending the request)
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct RunPayload {
    pub bench: Bench,
    pub prompt: PromptRequest,
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
    let openrouter = OpenRouter::new(config.api_key);

    while let Some(payload) = payloads.pop_front() {
        let bench = payload.bench.id.clone();

        // All chats will use this session as a base. The messages will get updated as responses come
        // in.
        let base_payload = payload.clone();

        // Pull out the messages for the chat.
        let mut bench_messages = payload
            .prompt
            .messages
            .clone()
            .into_iter()
            .flatten()
            .collect::<VecDeque<_>>();

        // All messages in the current session.
        let mut chat = Vec::new();

        let mut responses = Vec::new();

        while let Some(msg) = bench_messages.pop_front() {
            let mut request = base_payload.clone();
            // add the next benchmark message to the chat
            chat.push(msg);

            // clone the chat into the request prompt
            request.prompt.messages = Some(chat.clone());

            // generate a completion
            let response = complete(&openrouter, request, &model, &bench).await?;

            // If we get an assistant message back as the response, add it to the chat.
            if let Some(choice) = response.choices.last()
                && let Choice::NonStreaming(choice) = choice
            {
                let new_msg = extract_assistant_message(choice);

                chat.push(new_msg);
            }

            // push the complete response for eval
            responses.push(response);
        }

        let bench_result = BenchResult {
            hash: payload.prompt.prompt_hash(),
            bench: bench.clone(),
            request: payload.prompt,
            responses,
        };

        config
            .results_tx
            .send(ResultWriterCmd::SaveResult(bench_result))
            .unwrap();
    }

    Ok(())
}

/// Converts an assistant response into an assistant request message in a chat session.
fn extract_assistant_message(
    choice: &openrouter::completions::response::NonStreamingChoice,
) -> Message {
    Message::Assistant {
        content: choice
            .message
            .content
            .as_ref()
            .map(|msg| Content::Plain(msg.clone())),
        name: None,
        tool_calls: None,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompletionErrorWrapper {
    #[error("openrouter error {_0}")]
    OpenRouter(#[from] openrouter::error::Error),
}

/// Run a completion request.
async fn complete(
    api: &OpenRouter,
    request: RunPayload,
    model: &ModelId,
    bench: &BenchId,
) -> Result<Response, Report<CompletionError>> {
    tracing::info!(model=%model, bench=%model, "sending completion request");

    // Add 1 minute timeout
    let timeout_duration = std::time::Duration::from_secs(60);

    match tokio::time::timeout(
        timeout_duration,
        api.chat_completion(request.prompt.make_openrouter_request()),
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
                    Err(e)
                        .map_err(CompletionErrorWrapper::OpenRouter)
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

pub async fn run(config: RunConfig, model: ModelId, payloads: VecDeque<RunPayload>) {
    if let Err(e) = run_impl(config, model.clone(), payloads).await {
        tracing::error!(model=%model, err=?e, "an error occurred while processing a request");
    }
}
