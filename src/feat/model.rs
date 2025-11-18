use std::{collections::HashMap, path::Path};

use derive_more::{Display, FromStr};
use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

#[derive(
    Display,
    Default,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    FromStr,
)]
#[display("{_0}")]
pub struct ModelId(pub String);

impl ModelId {
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn contains(&self, other: &ModelId) -> bool {
        self.0.contains(other.as_str())
    }
}

/// Models to use in a bench/eval.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SelectedModels {
    models: Vec<ModelId>,
}

impl SelectedModels {
    pub const fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.models.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ModelId> {
        self.models.iter()
    }
}

impl<'a> IntoIterator for &'a SelectedModels {
    type Item = &'a ModelId;
    type IntoIter = std::slice::Iter<'a, ModelId>;

    fn into_iter(self) -> Self::IntoIter {
        self.models.iter()
    }
}

impl IntoIterator for SelectedModels {
    type Item = ModelId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.models.into_iter()
    }
}

impl Extend<ModelId> for SelectedModels {
    fn extend<T: IntoIterator<Item = ModelId>>(&mut self, iter: T) {
        self.models.extend(iter);
    }
}

impl FromIterator<ModelId> for SelectedModels {
    fn from_iter<T: IntoIterator<Item = ModelId>>(iter: T) -> Self {
        SelectedModels {
            models: iter.into_iter().collect(),
        }
    }
}

/// Models from the config file.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Models {
    models: Vec<ModelId>,
    model_groups: HashMap<String, Vec<ModelId>>,
}

impl IntoIterator for Models {
    type Item = ModelId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.models.into_iter()
    }
}

impl Extend<ModelId> for Models {
    fn extend<T: IntoIterator<Item = ModelId>>(&mut self, iter: T) {
        self.models.extend(iter);
    }
}

impl FromIterator<ModelId> for Models {
    fn from_iter<T: IntoIterator<Item = ModelId>>(iter: T) -> Self {
        Models {
            models: iter.into_iter().collect(),
            model_groups: HashMap::default(),
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

    pub const fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    pub fn iter_groups(&self) -> std::collections::hash_map::Iter<'_, String, Vec<ModelId>> {
        self.model_groups.iter()
    }
}
