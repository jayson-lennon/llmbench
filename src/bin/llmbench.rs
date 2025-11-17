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
async fn main() -> Result<(), Report<AppError>> {
    init::init_tracing();
    dotenv().unwrap();
    let args = Args::parse();

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
