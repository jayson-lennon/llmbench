use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Models {
    models: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("a ModelsError occurred")]
pub struct ModelsError;

impl Models {
    pub async fn load_from<P>(path: P) -> Result<Models, Report<ModelsError>>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new()
            .create(false)
            .read(true)
            .open(&path)
            .await
            .change_context(ModelsError)
            .attach_with(|| {
                format!("failed to open model file at '{}'", path.as_ref().display())
            })?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .await
            .change_context(ModelsError)
            .attach_with(|| {
                format!("failed to read model file at '{}'", path.as_ref().display())
            })?;

        toml::from_str(&buf)
            .change_context(ModelsError)
            .attach_with(|| {
                format!(
                    "failed to parse toml for model file at '{}'",
                    path.as_ref().display()
                )
            })
    }
}
