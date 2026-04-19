# LLM Bench

A CLI tool for running 1-shot benchmarks for Large Language Models via the [OpenRouter](https://openrouter.ai/) API.

## Features

- Run benchmarks on multiple models and tasks concurrently.
- AGENTS.md support
- Filter and select specific benchmarks, models, or model groups.
- Doesn't re-run completed tasks.
- Evaluate responses and report pass/fail with token usage and cost breakdown.
- Per-agent and grand total summary tables with sorting by any column.
- Group models into categories via config for easier model selection.

## Usage

This software is designed to run from source:

1. Install Rust if not already installed: [rustup.rs](https://rustup.rs)
2. Clone the repo and build:

```
git clone https://github.com/jayson-lennon/llmbench.git
cd llmbench
cargo build --release
```

3. Run with `./target/release/llmbench` or use `cargo run --release -- YOUR_COMMANDS_HERE`

## Configuration

The config file comes pre-loaded with some of the models available on OpenRouter at the time of publishing.

Update the `config.toml` with the models that you are interested in testing. `llmbench` runs the benchmark against all models listed unless a CLI flag is used to bench a specific model. Model IDs are from [the model list](https://openrouter.ai/models) at OpenRouter:

```toml
models = [
    "anthropic/claude-sonnet-4.5",
    "deepseek/deepseek-chat-v3.1",
    "google/gemini-2.5-pro",
    "meta-llama/llama-4-maverick",
]

[model_groups]
cheap_stuff = [
    "google/gemini-2.5-flash-lite",
    "inception/mercury-2",
    "meta-llama/llama-3.3-70b-instruct",
]
```

- Set `OPENROUTER_API_KEY` environment variable for authentication. See the `.env.example` file.

## Usage

### Benchmarking

Run all benchmarks on all models (defaults to 3 runs per model per benchmark):

```
llmbench bench
```

Run specific benchmarks and models:

```
llmbench bench -m "x-ai/grok-4" -m "qwen/qwen3-coder" --n-runs 2 logic/seating
```

Run benchmarks composed with an agent `.md` file:

```
llmbench bench -a "forge-mythos"
```

The `-a` flag accepts a glob pattern matching `.md` files in `src/agents_md/`. When specified, each benchmark is run twice: once as a baseline (no agent) and once with the agent content prepended to the prompt.

### Evaluation

Evaluate all results:

```
llmbench eval
```

Evaluate results, filtering by model and bench:

```
llmbench eval -m "gpt-4o-mini" -b logic/seating
```

Evaluate with agent filtering:

```
llmbench eval -a "forge-mythos"
```

Use `-c` / `--condensed` to suppress individual response output:

```
llmbench eval -c
```

#### Sorting

Use `--sort` / `-s` to sort the output by a specific column:

```
llmbench eval -s model
llmbench eval -s agent
llmbench eval -s cost
```

Available sort columns: `bench` (default), `model`, `agent`, `in`, `out`, `reason`, `cost`, `cost-delta`.

#### Output

Running `eval` prints three sections:

**1. Detail rows**: one row per bench + model + agent combination:

```
 model  | bench          | AGENTS.md    | result   | passed | in     | out   | reason | cost/run ($USD) | % cost Δ
--------+----------------+--------------+----------+--------+--------+-------+--------+-----------------+---------
 model-a| logic/seating  |              | ✅ Pass  | 4/4    | 521    | 5981  | 5957   | $0.001132993    | -
 model-a| logic/seating  | forge-mythos | ❌ Fail  | 0/4    | 16496  | 1925  | -      | $0.000721334    | +263.27%
```

**2. Per-agent summary**: aggregated totals grouped by `AGENTS.md` file, with pass rate, token usage, total cost, and median % cost delta:

```
        |          | AGENTS.md    | % pass  | passed  | in     | out    | reason | total cost ($USD) | med % cost Δ
--------+----------+--------------+---------+---------+--------+--------+--------+-------------------+--------------
        |          | (baseline)   | 86.88%  | 245/282 | 62784  | 77550  | 55456  | $0.058984334      | -
        |          | forge-mythos | 88.57%  | 248/280 | 118483 | 58734  | 42614  | $0.133469179      | +432.74%
```

The `(baseline)` row shows aggregated stats for all runs without an `AGENTS.md`. Agent rows show how each agent performed overall. The `med % cost Δ` column shows the median cost difference compared to the baseline.

**3. Grand totals**: the overall summary across all rows:

```
        |          | Grand totals | 87.72%  | 493/562 | 1247623| 136284 | 98070  | $0.192453513      | +432.74%
```

Color coding is used throughout: cyan for pass, red for fail.

The response section (shown in non-condensed mode) displays LLM responses using this format:

```
<run #>R<turn #>: <response>
```

For example, if a benchmark has been run twice for a specific model, you'll see output like this:

```
1R1: <output from the first run>
2R1: <output from the second run>
```

### Resetting Results

Delete all results for a specific benchmark (all agents, all models):

```
llmbench reset logic/seating
```

This removes both baseline and agent-composed results for the given bench ID. The argument is an exact match (no globs).

## Adding Benchmarks

Benchmarks are defined as `.md` files in `src/prompts/` using TOML frontmatter. The file path (minus `.md`) becomes the bench ID.

### File format

```markdown
---
expected = "charlie"
---

Your prompt goes here.
```

The `expected` field in the frontmatter is the expected answer. The evaluator normalizes both the expected value and the model's response by lowercasing, stripping chat tags, and keeping only alphanumeric characters before comparison.

### Directory structure

```
src/prompts/
  logic/
    seating.md      → bench ID: logic/seating
    schedule.md     → bench ID: logic/schedule
  decision_making/
    triage.md       → bench ID: decision_making/triage
  extraction/
    flight.md       → bench ID: extraction/flight
```

### Adding agent prompts

Agent prompts are `.md` files in `src/agents_md/`. The filename (minus `.md`) becomes the agent name. When an agent is specified, its content is prepended to the benchmark prompt:

```
src/agents_md/
  forge-mythos.md   → agent name: forge-mythos
```

When composed, the resulting prompt is:

```
<agent content>
---
<benchmark prompt>
```

## License

[GPL-3.0](https://www.gnu.org/licenses/gpl-3.0.txt)
