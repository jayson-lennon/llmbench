use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use indicatif::ProgressBar;
use serde::Serialize;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::feat::bench::BenchResult;

pub type ResultReceiver = tokio::sync::mpsc::UnboundedReceiver<ResultWriterCmd>;
pub type ResultSender = tokio::sync::mpsc::UnboundedSender<ResultWriterCmd>;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ResultWriterCmd {
    SaveResult(BenchResult),
    /// Tell the writer that a job failed. The writer should update the remaining jobs.
    JobError,
    Quit,
}

#[tracing::instrument(skip(path, rx, pb))]
pub async fn spawn_result_writer<P>(
    path: P,
    total_requests: usize,
    pb: ProgressBar,
    mut rx: ResultReceiver,
) where
    P: Into<PathBuf>,
{
    let mut completed = 0;
    let path = path.into();
    while let Some(cmd) = rx.recv().await {
        match cmd {
            ResultWriterCmd::SaveResult(result) => {
                completed += 1;
                pb.inc(1);
                tracing::trace!(
                    remaining = (total_requests.saturating_sub(completed)),
                    total = total_requests,
                    "received result"
                );
                if let Err(e) = write_to_ndjson(path.clone(), &result).await {
                    tracing::error!(err=?e, "failed to save results");
                } else {
                    tracing::trace!(bench=%result.bench, model=%result.model, "wrote result");
                }
            }
            ResultWriterCmd::JobError => {
                completed += 1;
                pb.inc(1);
            }
            ResultWriterCmd::Quit => return,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("a SaveError occurred")]
pub struct SaveError;

#[tracing::instrument(skip_all, err)]
pub async fn write_to_ndjson<P, S>(path: P, result: &S) -> Result<(), Report<SaveError>>
where
    P: AsRef<Path>,
    S: Serialize,
{
    let result = serde_json::to_string(result)
        .change_context(SaveError)
        .attach("failed to serialized results into json")?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .await
        .change_context(SaveError)
        .attach("failed to open results file")?;
    file.write_all(result.as_bytes())
        .await
        .change_context(SaveError)
        .attach("failed to write results")?;
    file.write_all("\n".as_bytes())
        .await
        .change_context(SaveError)
        .attach("failed to write newline")?;
    Ok(())
}
