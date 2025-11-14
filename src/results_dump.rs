use std::{io::ErrorKind, path::Path};

use error_stack::{Report, ResultExt};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use crate::promptresult::PromptResult;

#[derive(Debug, Clone)]
pub struct ResultsDump {
    pub results: Vec<PromptResult>,
}

#[derive(Debug, thiserror::Error)]
#[error("a ResultsDump error occurred")]
pub struct ResultsDumpError;

impl ResultsDump {
    pub async fn load<P>(path: P) -> Result<Self, Report<ResultsDumpError>>
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
                ErrorKind::NotFound => return Ok(Self { results: vec![] }),
                _ => {
                    return Err(e).change_context(ResultsDumpError).attach_with(|| {
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
            .change_context(ResultsDumpError)
            .attach_with(|| {
                format!(
                    "failed to read results file at '{}'",
                    path.as_ref().display()
                )
            })?;

        let mut results = Vec::new();
        for result in buf.lines() {
            let result: PromptResult = serde_json::from_str(result)
                .change_context(ResultsDumpError)
                .attach("deserialization eror")?;
            results.push(result);
        }
        Ok(Self { results })
    }
}

impl IntoIterator for ResultsDump {
    type Item = PromptResult;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.into_iter()
    }
}

impl<'a> IntoIterator for &'a ResultsDump {
    type Item = &'a PromptResult;
    type IntoIter = std::slice::Iter<'a, PromptResult>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.iter()
    }
}

impl Extend<PromptResult> for ResultsDump {
    fn extend<T: IntoIterator<Item = PromptResult>>(&mut self, iter: T) {
        self.results.extend(iter);
    }
}

impl FromIterator<PromptResult> for ResultsDump {
    fn from_iter<T: IntoIterator<Item = PromptResult>>(iter: T) -> Self {
        Self {
            results: Vec::from_iter(iter),
        }
    }
}
