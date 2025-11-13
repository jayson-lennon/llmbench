use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    bench_loader::{BenchId, Benches},
    init,
    models::Models,
    promptrequest::PromptRequest,
    promptresult::PromptResult,
    results_dump::ResultsDump,
};
use openrouter::{
    OpenRouter,
    completions::request::{Content, Message},
};
use serde::Serialize;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

// TODO:
// - figure out what benches still need to be ran
// - spawn a task per model
// - execute the prompt by sending
// - handle multi-turn prompts

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The benches to run. Runs all benches by default.
    benches: Vec<String>,

    /// Specify which models to bench. Runs on all models by default.
    #[arg(short, long)]
    models: Vec<String>,

    /// OpenRouter API key. (Prefer OPENROUTER_API_KEY env variable for security)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Path to bench dir
    #[arg(long, default_value = "bench")]
    bench_path: PathBuf,

    /// Path to bench results
    #[arg(short, long, default_value = "results.ndjson")]
    results: PathBuf,
}

#[derive(Debug, thiserror::Error)]
#[error("an application error occurred")]
struct AppError;

#[tokio::main]
async fn main() -> Result<(), Report<AppError>> {
    init::init_tracing();
    dotenv().unwrap();
    let args = Args::parse();

    let api_key = {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            key
        } else {
            let Some(key) = args.api_key.clone() else {
                tracing::error!("OPENROUTER_API_KEY env variable or CLI arg required");
                return Err(Report::from(AppError));
            };
            key
        }
    };

    let benches = {
        // make sure all benches follow the correct format. bail otherwise
        let bench_ids = match get_bench_ids_to_run(&args) {
            Ok(ids) => ids,
            Err(er) => return Err(er),
        };

        let benches = Benches::new(args.bench_path)
            .await
            .change_context(AppError)
            .attach("failed to load benches")?;
        if bench_ids.is_empty() {
            benches
        } else {
            // bail if we cant find a bench that the user provided
            for id in &bench_ids {
                if !benches.contains(id) {
                    tracing::error!(id=%id, "bench not found");
                    return Err(Report::from(AppError));
                }
            }
            benches
                .into_iter()
                .filter(|bench| bench_ids.contains(&bench.id))
                .collect()
        }
    };

    let openrouter = OpenRouter::new(api_key);

    let models = {
        let models = Models::load_from(args.config)
            .await
            .change_context(AppError)
            .attach("failed to load model list")?;
        if args.models.is_empty() {
            models
        } else {
            Models::from_iter(args.models)
        }
    };

    let results = ResultsDump::load(&args.results)
        .await
        .change_context(AppError)
        .attach("failed to load existing results")?;

    dbg!(results, models, benches);
    return Ok(());

    let request = PromptRequest {
        run_number: 2,
        messages: Some(vec![Message::User {
            content: Content::Plain("Hello, how are you today?".to_string()),
            name: None,
            cache_control: None,
        }]),
        model: "google/gemma-3-27b-it:free".to_string(),
        ..Default::default()
    };

    let hash = request.prompt_hash();

    match openrouter
        .chat_completion(request.make_openrouter_request())
        .await
    {
        Ok(response) => {
            let obj = PromptResult {
                hash,
                category: todo!(),
                bench: todo!(),
                request,
                responses: vec![response],
            };
            write_to_ndjson(&args.results, &obj)
                .await
                .change_context(AppError)?;
            // match openrouter.generation(&response.id).await {
            //     Ok(info) => {
            //         dbg!(&info);
            //     }
            //     Err(e) => eprintln!("failed to get generation info {e:?}"),
            // }
        }
        Err(e) => {
            tracing::error!(err=?e, "failed to get chat completion");
        }
    }
    Ok(())
}

fn get_bench_ids_to_run(args: &Args) -> Result<Vec<BenchId>, Report<AppError>> {
    let bench_ids = args
        .benches
        .iter()
        .map(|bench| BenchId::from_str(bench))
        .collect::<Vec<_>>();
    let mut errors = false;
    for id in &bench_ids {
        if let Err(e) = id {
            errors = true;
            tracing::error!(err=?e);
        }
    }
    if errors {
        return Err(Report::from(AppError)).attach("error parsing bench ids");
    }
    Ok(bench_ids.into_iter().flatten().collect::<Vec<_>>())
}

#[derive(Debug, thiserror::Error)]
#[error("a SaveError occurred")]
struct SaveError;

async fn write_to_ndjson<P, S>(path: P, result: &S) -> Result<(), Report<SaveError>>
where
    P: AsRef<Path>,
    S: Serialize,
{
    let result = serde_json::to_string(result)
        .change_context(SaveError)
        .attach("failed to serialized results into json")?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .await
        .change_context(SaveError)
        .attach("failed to open results file")?;
    file.write_all(result.as_bytes())
        .await
        .change_context(SaveError)
        .attach("failed to write results")?;
    file.write_all("\n".as_bytes())
        .await
        .change_context(SaveError)
        .attach("failed to write newline")?;
    Ok(())
}
