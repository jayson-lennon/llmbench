use std::collections::HashMap;

use crate::{
    feat::bench::BenchId,
    feat::evaluator::score::{BenchModelKey, ScoredBench, Scores},
    feat::model::ModelId,
};

use super::display::pct_cost_diff;

#[derive(Clone, Default, Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct EntryAgg {
    pub(super) prompt_tokens: u32,
    pub(super) completion_tokens: u32,
    pub(super) reasoning_tokens: u32,
    pub(super) cost: f64,
    pub(super) passed: usize,
    pub(super) runs: usize,
}

impl EntryAgg {
    #[allow(clippy::cast_precision_loss)]
    pub(super) fn cost_per_run(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.cost / self.runs as f64
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(super) struct AggregatedScores {
    entries: HashMap<BenchModelKey, EntryAgg>,
}

impl AggregatedScores {
    pub(super) fn build(scores: &Scores) -> Self {
        let mut agg = Self::default();
        for score in scores.values().flatten() {
            agg.add(score);
        }
        agg
    }

    fn add(&mut self, bench: &ScoredBench) {
        let key = BenchModelKey {
            bench_id: bench.result.bench.clone(),
            model_id: bench.result.model.clone(),
        };
        let entry = self.entries.entry(key).or_default();
        entry.runs += 1;
        entry.passed += usize::from(bench.score.passed);
        for res in &bench.result.responses {
            if let Some(usage) = &res.usage {
                entry.prompt_tokens += usage.prompt_tokens;
                entry.completion_tokens += usage.completion_tokens;
                entry.reasoning_tokens += usage
                    .completion_tokens_details
                    .as_ref()
                    .map_or(0, |d| d.reasoning_tokens);
                entry.cost += usage.cost.unwrap_or_default();
            }
        }
    }

    pub(super) fn for_key(&self, key: &BenchModelKey) -> &EntryAgg {
        static DEFAULT: EntryAgg = EntryAgg {
            prompt_tokens: 0,
            completion_tokens: 0,
            reasoning_tokens: 0,
            cost: 0.0,
            passed: 0,
            runs: 0,
        };
        self.entries.get(key).unwrap_or(&DEFAULT)
    }

    /// Look up the baseline (no-agent) entry for a given bench base name + model.
    pub(super) fn baseline_cost_per_run(
        &self,
        bench_base: &str,
        model_id: &str,
    ) -> Option<f64> {
        let key = BenchModelKey {
            bench_id: BenchId(bench_base.to_string()),
            model_id: ModelId(model_id.to_string()),
        };
        let entry = self.entries.get(&key)?;
        (entry.runs > 0).then_some(entry.cost_per_run())
    }

    /// Compute per-agent aggregation groups.
    /// Returns one `AgentGroup` per distinct agent suffix, sorted
    /// `(baseline)` first, then alphabetical.
    pub(super) fn per_agent_totals(&self) -> Vec<AgentGroup> {
        let mut groups: HashMap<Option<String>, AgentGroup> = HashMap::new();

        for (key, entry) in self.entries.iter() {
            let (_base, agent_suffix) = split_agent_suffix(&key.bench_id.0);
            let label_key = agent_suffix.map(String::from);
            let group = groups.entry(label_key).or_insert_with(|| AgentGroup {
                label: agent_suffix
                    .map(String::from)
                ,
                totals: Totals::default(),
                cost_diffs: Vec::new(),
            });

            group.totals.passed += entry.passed;
            group.totals.runs += entry.runs;
            group.totals.prompt_tokens += entry.prompt_tokens;
            group.totals.completion_tokens += entry.completion_tokens;
            group.totals.reasoning_tokens += entry.reasoning_tokens;
            group.totals.cost += entry.cost;

            // Collect cost diffs for agent entries (not baseline)
            if agent_suffix.is_some() {
                if let Some(base_cpp) = self.baseline_cost_per_run(_base, &key.model_id.0) {
                    let pct = pct_cost_diff(entry.cost_per_run(), base_cpp);
                    group.cost_diffs.push(pct);
                }
            }
        }

        let mut result: Vec<AgentGroup> = groups.into_values().collect();
        result.sort_by(|a, b| {
            match (&a.label, &b.label) {
                (None, _) => std::cmp::Ordering::Less,
                (_, None) => std::cmp::Ordering::Greater,
                (Some(la), Some(lb)) => la.cmp(lb),
            }
        });
        result
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct Totals {
    pub(super) passed: usize,
    pub(super) runs: usize,
    pub(super) prompt_tokens: u32,
    pub(super) completion_tokens: u32,
    pub(super) reasoning_tokens: u32,
    pub(super) cost: f64,
}

impl Totals {
    #[allow(clippy::cast_precision_loss)]
    pub(super) fn pct_pass(&self) -> f64 {
        self.passed as f64 / self.runs as f64
    }
}

/// Per-agent aggregation group.
#[derive(Debug, Clone)]
pub(super) struct AgentGroup {
    /// `None` means baseline (no agent).
    pub(super) label: Option<String>,
    pub(super) totals: Totals,
    pub(super) cost_diffs: Vec<f64>,
}

impl AgentGroup {
    /// Display label: `"(baseline)"` for `None`, otherwise the agent name.
    pub(super) fn display_label(&self) -> &str {
        self.label.as_deref().unwrap_or("(baseline)")
    }

    /// Compute median of collected cost diffs, or `None` if empty.
    pub(super) fn median_cost_diff(&mut self) -> Option<f64> {
        if self.cost_diffs.is_empty() {
            return None;
        }
        self.cost_diffs
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = self.cost_diffs.len() / 2;
        Some(if self.cost_diffs.len().is_multiple_of(2) {
            f64::midpoint(self.cost_diffs[mid - 1], self.cost_diffs[mid])
        } else {
            self.cost_diffs[mid]
        })
    }
}

/// Split `category/name+agent` into (`category/name`, Some(`agent`)).
/// If no `+`, returns (`id`, None).
pub(super) fn split_agent_suffix(id: &str) -> (&str, Option<&str>) {
    match id.rsplit_once('+') {
        Some((base, agent)) => (base, Some(agent)),
        None => (id, None),
    }
}
