use glob::glob;
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use error_stack::{Report, ResultExt};

use crate::feat::bench::{BenchError, BenchId, BenchResult};

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

/// A specific benchmark
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Bench {
    pub id: BenchId,
    pub system_prompt: Option<String>,
    pub prompts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Benches {
    pub benches: Vec<Bench>,
}

impl Benches {
    pub fn contains(&self, id: &BenchId) -> bool {
        self.benches.iter().any(|bench| &bench.id == id)
    }

    pub async fn new<P>(path: P) -> Result<Self, Report<BenchError>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let mut benches = Vec::new();

        let mut stream = tokio::fs::read_dir(path)
            .await
            .change_context(BenchError)
            .attach_with(|| format!("failed to access bench dir '{}'", path.display()))?;

        while let Some(entry) = stream
            .next_entry()
            .await
            .change_context(BenchError)
            .attach_with(|| format!("failed to read bench dir '{:?}'", path.display()))?
        {
            let category = entry.path();

            let bench_dirs = get_directory_names(&category)
                .await
                .change_context(BenchError)
                .attach("failed to get bench directories")?;

            let category = category
                .file_name()
                .ok_or(Report::from(BenchError))
                .attach_with(|| format!("empty bench category for '{}'", category.display()))?
                .to_string_lossy();

            for dir in bench_dirs {
                let name = dir
                    .file_name()
                    .ok_or(Report::from(BenchError))
                    .attach_with(|| format!("failed to get bench name for '{}'", dir.display()))?
                    .to_string_lossy();
                let bench = load_bench(category.clone(), name.clone(), &dir).await?;
                benches.push(bench);
            }
        }

        Ok(Self { benches })
    }

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

async fn load_bench<P, T1, T2>(category: T1, name: T2, path: P) -> Result<Bench, Report<BenchError>>
where
    P: AsRef<Path>,
    T1: Into<String>,
    T2: Into<String>,
{
    let mut prompts = Vec::new();

    let prompt_glob = format!("{}/prompt*.md", path.as_ref().display());
    for entry in glob(&prompt_glob).unwrap() {
        match entry {
            Ok(path) => {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .change_context(BenchError)
                    .attach_with(|| {
                        format!("failed to read bench prompt file '{}'", path.display())
                    })?;
                prompts.push(content);
            }
            Err(e) => tracing::error!(err=?e, "Error reading glob entry"),
        }
    }

    let system_prompt = {
        let mut path = PathBuf::from(path.as_ref());
        path.push("system.md");
        match OpenOptions::new().read(true).open(path).await {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)
                    .await
                    .change_context(BenchError)
                    .attach("failed to read system.md")?;
                Some(buf)
            }
            Err(err) => match err.kind() {
                ErrorKind::NotFound => None,
                _ => {
                    return Err(err)
                        .change_context(BenchError)
                        .attach("failed to read system.md");
                }
            },
        }
    };

    let category = category.into();
    let name = name.into();
    Ok(Bench {
        id: BenchId::from_str(&format!("{}/{}", &category, &name)).unwrap(),
        system_prompt,
        prompts,
    })
}

async fn get_directory_names<P>(path: P) -> Result<Vec<PathBuf>, tokio::io::Error>
where
    P: AsRef<Path>,
{
    let mut dir_names = Vec::new();

    let mut stream = tokio::fs::read_dir(path).await?;

    while let Ok(Some(entry)) = stream.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            dir_names.push(path);
        }
    }

    Ok(dir_names)
}
