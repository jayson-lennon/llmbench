pub mod bench;
pub mod eval;

use std::path::PathBuf;

pub use bench::BenchArgs;
use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use error_stack::{Report, ResultExt};
pub use eval::EvalArgs;

use crate::{
    error::ErrContext,
    feat::model::{ModelId, Models, SelectedModels},
};

#[derive(Debug, thiserror::Error)]
#[error("a CliError occurred")]
pub struct CliError;

#[derive(Parser, Debug)]
pub struct SharedArgs {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Path to bench results
    #[arg(short, long, default_value = "results.ndjson")]
    pub results: PathBuf,

    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}

async fn select_models(
    models: &[ModelId],
    groups: &[String],
    shared_args: &SharedArgs,
) -> Result<SelectedModels, Report<[CliError]>> {
    let mut error: Option<Report<[CliError]>> = None;

    let models = match (models.is_empty(), groups.is_empty()) {
        (true, true) => {
            // Both empty: select all
            let all_models = Models::load_from(&shared_args.config)
                .await
                .change_context(CliError)
                .attach("failed to load models from config file")?;
            all_models.into_iter().collect::<SelectedModels>()
        }
        (true, false) => {
            // No models selected, but groups are selected: load from config and filter groups
            let all_models = Models::load_from(shared_args.config.clone())
                .await
                .change_context(CliError)
                .attach("failed to load models from config file")?;

            for group in groups {
                if !all_models
                    .iter_groups()
                    .any(|(available, _)| available == group)
                {
                    let err = Report::from(CliError)
                        .attach("group not found")
                        .attach(ErrContext(format!("group name = {group}")));
                    if let Some(error) = error.as_mut() {
                        error.push(err);
                    } else {
                        error = Some(err.expand());
                    }
                }
            }
            if let Some(error) = error {
                return Err(error);
            }
            all_models
                .iter_groups()
                .filter(|(group, _)| groups.contains(group))
                .flat_map(|(_, models)| models.iter().cloned())
                .collect::<SelectedModels>()
        }
        (false, true) => {
            // Models only: select specified models
            models.iter().cloned().collect::<SelectedModels>()
        }
        (false, false) => {
            // Both specified: unreachable (Clap enforces mutual exclusion)
            unreachable!("programming error: cannot select both individual models and model groups")
        }
    };
    Ok(models)
}
