use derive_more::Display;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use error_stack::{Report, ResultExt};

#[derive(Debug, thiserror::Error)]
#[error("a BenchError occurred")]
pub struct BenchError;

#[derive(Display, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[display("{_0}")]
pub struct BenchId(pub String);

impl BenchId {
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromStr for BenchId {
    type Err = Report<BenchError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((category, name)) = s.split_once('/') {
            if category.is_empty() {
                return Err(Report::from(BenchError)).attach("missing category from bench id");
            }
            if name.is_empty() {
                return Err(Report::from(BenchError)).attach("missing name from bench id");
            }
            Ok(Self(s.to_string()))
        } else {
            Err(Report::from(BenchError)).attach("invalid id format (must be category/benchname)")
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
