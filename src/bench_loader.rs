use glob::glob;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use error_stack::{Report, ResultExt};

#[derive(Debug, thiserror::Error)]
#[error("an BenchError occurred")]
pub struct BenchError;

/// A specific benchmark
#[derive(Debug, Clone)]
pub struct Bench {
    pub category: String,
    pub name: String,
    pub system_prompt: Option<String>,
    pub prompts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Benches {
    pub benches: Vec<Bench>,
}

impl Benches {
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
            Err(e) => println!("Error reading glob entry: {}", e),
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

    Ok(Bench {
        category: category.into(),
        name: name.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reads_dir() {
        let benches = Benches::new("prompts").await.unwrap();
        dbg!(&benches);
    }
}
