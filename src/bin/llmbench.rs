use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    feat::{
        self,
        cli::{BenchArgs, EvalArgs, SharedArgs},
    },
    init,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    Bench(BenchArgs),
    Eval(EvalArgs),
}

/// A simple LLM benchmark tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[command(flatten)]
    shared: SharedArgs,
}

#[derive(Debug, thiserror::Error)]
#[error("an application error occurred")]
struct AppError;

#[tokio::main]
#[tracing::instrument(err)]
async fn main() -> Result<(), Report<AppError>> {
    init::init_error_stack();

    dotenv()
        .or_else(|e| match e {
            dotenvy::Error::Io(io) if io.kind() == std::io::ErrorKind::NotFound => {
                Ok(PathBuf::from(".env"))
            }
            _ => Err(e),
        })
        .map_err(|e| {
            Report::from(e)
                .change_context(AppError)
                .attach("failed to load .env file")
        })?;

    let args = Args::parse();
    init::init_tracing(args.shared.verbosity);

    match args.command {
        Command::Bench(bench) => feat::cli::bench::run(bench, args.shared)
            .await
            .change_context(AppError)?,
        Command::Eval(eval) => feat::cli::eval::run(eval, args.shared)
            .await
            .change_context(AppError)?,
    }

    Ok(())
}
