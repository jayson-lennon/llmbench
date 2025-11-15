use std::{io::ErrorKind, path::Path};

use error_stack::{Report, ResultExt};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use crate::bench_result::BenchResult;

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
