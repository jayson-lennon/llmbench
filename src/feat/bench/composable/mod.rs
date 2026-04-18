use std::path::Path;

use error_stack::{Report, ResultExt};
use serde::Deserialize;

use crate::feat::{
    bench::{
        Bench, BenchId, Benches,
        BenchResultRequestExt, BenchResultResponseExt,
        StringBenchExt, user_message, GetMessageExt,
    },
    completion::{PromptRequest, worker::CompletionError},
    evaluator::{Evaluator, Score},
};

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ComposableError {
    #[error("failed to discover benches")]
    Discover,
    #[error("failed to parse frontmatter")]
    Frontmatter,
    #[error("failed to load agents.md")]
    LoadAgents,
}

// ── Frontmatter ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct BenchFrontmatter {
    expected: String,
}

// ── Discovered bench ────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DiscoveredBench {
    pub id: String,
    pub expected: String,
    pub prompt: String,
}

// ── Parsing ─────────────────────────────────────────────────────────────────

/// Parse TOML frontmatter delimited by `---` from file content.
/// Returns (frontmatter, prompt_body).
fn parse_frontmatter(content: &str) -> Result<(BenchFrontmatter, &str), Report<ComposableError>> {
    let content = content.strip_prefix("---").ok_or_else(|| {
        Report::new(ComposableError::Frontmatter).attach("file must start with ---")
    })?;

    let (toml_str, body) = content.split_once("---").ok_or_else(|| {
        Report::new(ComposableError::Frontmatter)
            .attach("file must contain a closing --- delimiter after frontmatter")
    })?;

    let fm: BenchFrontmatter =
        toml::from_str(toml_str.trim()).change_context(ComposableError::Frontmatter)?;

    Ok((fm, body.trim_start_matches('\n')))
}

// ── Discovery ───────────────────────────────────────────────────────────────

/// Walk `dir` recursively for `.md` files, parse frontmatter, filter by glob pattern.
/// If `pattern` contains glob metacharacters it is treated as a glob; otherwise exact match.
/// The bench ID is derived from the relative path (strip dir prefix + `.md` suffix).
///
/// # Panics
/// Panics if a discovered file path cannot be made relative to `dir`.
pub fn discover_benches(
    dir: &Path,
    pattern: &str,
) -> Result<Vec<DiscoveredBench>, Report<ComposableError>> {
    let mut benches = Vec::new();

    let entries = collect_md_files(dir).change_context(ComposableError::Discover)?;

    for path in entries {
        let relative = path
            .strip_prefix(dir)
            .expect("path is under dir")
            .to_string_lossy()
            .to_string();
        let bench_id = relative.trim_end_matches(".md").to_string();

        if !matches_pattern(&bench_id, pattern) {
            continue;
        }

        let content =
            std::fs::read_to_string(&path).change_context(ComposableError::Discover)?;
        let (fm, prompt) = parse_frontmatter(&content)?;

        benches.push(DiscoveredBench {
            id: bench_id,
            expected: fm.expected,
            prompt: prompt.to_string(),
        });
    }

    if benches.is_empty() && !pattern.is_empty() {
        return Err(Report::new(ComposableError::Discover)
            .attach("no benches matched the given pattern")
            .attach(format!("pattern: {pattern}")));
    }

    Ok(benches)
}

/// Walk `dir` for `.md` files, filter by pattern.
/// Returns (name, content) pairs where name is the filename minus `.md`.
///
/// # Panics
/// Panics if a discovered file path has no filename component.
pub fn discover_agents(
    dir: &Path,
    pattern: &str,
) -> Result<Vec<(String, String)>, Report<ComposableError>> {
    let mut agents = Vec::new();

    let entries = collect_md_files(dir).change_context(ComposableError::LoadAgents)?;

    for path in entries {
        let filename = path
            .file_name()
            .expect("file has a name")
            .to_string_lossy()
            .to_string();
        let name = filename.trim_end_matches(".md").to_string();

        if !matches_pattern(&name, pattern) {
            continue;
        }

        let content =
            std::fs::read_to_string(&path).change_context(ComposableError::LoadAgents)?;
        agents.push((name, content));
    }

    Ok(agents)
}

// ── Building ────────────────────────────────────────────────────────────────

/// Build the full bench set: baseline benches + cartesian product with agents.
pub fn build_bench_set(
    benches: &[DiscoveredBench],
    agents: &[(String, String)],
) -> Benches {
    let mut result = Vec::new();

    for bench in benches {
        // Baseline bench (no agents.md)
        result.push(build_single_bench(&bench.id, &bench.prompt, &bench.expected, None));

        // One bench per agents.md
        for (agent_name, agent_content) in agents {
            let id = format!("{}+{}", bench.id, agent_name);
            result.push(build_single_bench(&id, &bench.prompt, &bench.expected, Some(agent_content)));
        }
    }

    Benches { benches: result }
}

/// Build evaluators keyed by full bench ID (including `+agents_name` suffix).
pub fn build_evaluators(benches: &[DiscoveredBench], agents: &[(String, String)]) -> Vec<(BenchId, Evaluator)> {
    let mut evaluators = Vec::new();

    for bench in benches {
        evaluators.push((
            BenchId(bench.id.clone()),
            make_evaluator(&bench.expected),
        ));

        for (agent_name, _) in agents {
            let id = format!("{}+{}", bench.id, agent_name);
            evaluators.push((
                BenchId(id),
                make_evaluator(&bench.expected),
            ));
        }
    }

    evaluators
}

// ── Internals ───────────────────────────────────────────────────────────────

fn build_single_bench(
    id: &str,
    prompt: &str,
    _expected: &str,
    agents_md: Option<&str>,
) -> Bench {
    let full_prompt = match agents_md {
        Some(agents) => format!("{agents}\n---\n{prompt}"),
        None => prompt.to_string(),
    };
    let bench_id = BenchId(id.to_string());
    let id_owned = id.to_string();

    Bench::new(bench_id, move |api, ctx| {
        let full_prompt = full_prompt.clone();
        let id_owned = id_owned.clone();
        async move {
            let bench = BenchId(id_owned);
            let mut result = crate::feat::bench::BenchResult {
                hash: ctx.run_hash,
                bench: bench.clone(),
                model: ctx.model.clone(),
                requests: vec![],
                responses: vec![],
            };

            let request = PromptRequest::builder()
                .model(ctx.model.to_string())
                .messages(vec![user_message(&full_prompt)])
                .build()
                .save_to(&mut result);

            let _ = crate::feat::completion::worker::complete(
                &api,
                request.clone(),
                &ctx.model,
                &bench,
            )
            .await
            .map_err(|e| e.change_context(CompletionError))?
            .save_to(&mut result);

            Ok::<_, Report<CompletionError>>(result)
        }
    })
}

fn normalize_expected(s: &str) -> String {
    s.to_string()
        .lowercase()
        .remove_chat_tags()
        .alphanumeric_only()
        .trim()
        .to_string()
}

fn make_evaluator(expected: &str) -> Evaluator {
    let expected = normalize_expected(expected);
    Evaluator {
        bench: String::new(),
        eval: Box::new(move |responses: &[openrouter::completions::response::Choice]| -> Score {
            match responses {
                [a] => match a.get_message() {
                    Some(a) => {
                        let processed = a
                            .lowercase()
                            .remove_chat_tags()
                            .alphanumeric_only()
                            .trim()
                            .to_string();
                        Score::builder().passed(processed == expected).build()
                    }
                    _ => Score::fail(),
                },
                _ => Score::fail(),
            }
        }),
    }
}

fn matches_pattern(id: &str, pattern: &str) -> bool {
    glob::Pattern::new(pattern)
        .is_ok_and(|pat| pat.matches(id))
}

/// Recursively collect all `.md` files under `dir`.
fn collect_md_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    collect_md_files_recursive(dir, &mut files)?;
    Ok(files)
}

fn collect_md_files_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    Ok(())
}
