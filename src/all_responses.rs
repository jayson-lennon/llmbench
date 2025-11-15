use std::{io::ErrorKind, path::Path};

use error_stack::{Report, ResultExt};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use crate::promptresult::PromptResponse;

#[derive(Debug, Clone)]
pub struct AllResponses {
    pub inner: Vec<PromptResponse>,
}

#[derive(Debug, thiserror::Error)]
#[error("an AllResponsesError error occurred")]
pub struct AllResponsesError;

impl AllResponses {
    pub async fn load<P>(path: P) -> Result<Self, Report<AllResponsesError>>
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
                    return Err(e).change_context(AllResponsesError).attach_with(|| {
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
            .change_context(AllResponsesError)
            .attach_with(|| {
                format!(
                    "failed to read results file at '{}'",
                    path.as_ref().display()
                )
            })?;

        let mut results = Vec::new();
        for result in buf.lines() {
            let result: PromptResponse = serde_json::from_str(result)
                .change_context(AllResponsesError)
                .attach("deserialization eror")?;
            results.push(result);
        }
        Ok(Self { inner: results })
    }
}

impl IntoIterator for AllResponses {
    type Item = PromptResponse;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a AllResponses {
    type Item = &'a PromptResponse;
    type IntoIter = std::slice::Iter<'a, PromptResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl Extend<PromptResponse> for AllResponses {
    fn extend<T: IntoIterator<Item = PromptResponse>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}

impl FromIterator<PromptResponse> for AllResponses {
    fn from_iter<T: IntoIterator<Item = PromptResponse>>(iter: T) -> Self {
        Self {
            inner: Vec::from_iter(iter),
        }
    }
}
