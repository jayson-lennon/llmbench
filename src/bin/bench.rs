use std::path::{Path, PathBuf};

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use llmbench::{
    bench_loader::Benches, models::Models, promptrequest::PromptRequest,
    promptresult::PromptResult, results_dump::ResultsDump,
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
    /// OpenRouter API key (can also be set via OPENROUTER_API_KEY environment variable)
    #[arg(env = "OPENROUTER_API_KEY")]
    api_key: Option<String>,

    /// Path to slumber config
    #[arg(default_value = "config.toml")]
    config: PathBuf,

    /// Path to prompts dir
    #[arg(default_value = "prompts")]
    prompts: PathBuf,

    /// Path to bench results
    #[arg(default_value = "results.ndjson")]
    bench_results: PathBuf,
}

#[derive(Debug, thiserror::Error)]
#[error("an application error occurred")]
struct AppError;

#[tokio::main]
async fn main() -> Result<(), Report<AppError>> {
    dotenv().unwrap();
    let args = Args::parse();

    let Some(api_key) = args.api_key else {
        eprintln!("OPENROUTER_API_KEY env variable or CLI arg required");
        return Err(Report::from(AppError));
    };

    let openrouter = OpenRouter::new(api_key);

    let models = Models::load_from(args.config)
        .await
        .change_context(AppError)
        .attach("failed to load model list")?;

    let benches = Benches::new(args.prompts)
        .await
        .change_context(AppError)
        .attach("failed to load prompts")?;

    let results = ResultsDump::load(&args.bench_results)
        .await
        .change_context(AppError)
        .attach("failed to load existing results")?;
    dbg!(&results);
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
            write_to_ndjson(&args.bench_results, &obj)
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
            eprintln!("failed to get chat completion {e:?}");
        }
    }
    Ok(())
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
