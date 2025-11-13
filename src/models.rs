use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Models {
    models: Vec<String>,
}

impl IntoIterator for Models {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.models.into_iter()
    }
}

impl Extend<String> for Models {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.models.extend(iter);
    }
}

impl FromIterator<String> for Models {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Models {
            models: iter.into_iter().collect(),
        }
    }
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
