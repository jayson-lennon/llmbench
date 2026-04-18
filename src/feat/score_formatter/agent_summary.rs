use super::{
    aggregate::{AgentGroup, AggregatedScores},
    display::{format_pct_cost_diff, format_totals_passed},
    row::{ColWidths, Row},
};

/// Per-agent summary table printer.
pub(super) struct AgentSummary {
    groups: Vec<AgentGroup>,
}

impl AgentSummary {
    /// Build per-agent totals from aggregated scores.
    pub(super) fn build(agg: &AggregatedScores) -> Self {
        Self {
            groups: agg.per_agent_totals(),
        }
    }

    /// Print the per-agent summary table (header + separator + rows).
    pub(super) fn print(mut self, widths: &ColWidths) {
        let separator = Row::separator(widths);

        println!("{}", Row::summary_header(widths).render());
        println!("{}", separator.render_separator());

        for group in &mut self.groups {
            let label = group.display_label().to_string();
            let pct_pass = format!("{:.2}%", group.totals.pct_pass() * 100.0);
            let passed_str = format_totals_passed(&group.totals);
            let nreason_str = if group.totals.reasoning_tokens > 0 {
                group.totals.reasoning_tokens.to_string()
            } else {
                "-".to_string()
            };
            let cost_str = format!("${:.9}", group.totals.cost);

            let cost_diff_str = match group.median_cost_diff() {
                Some(median) => format_pct_cost_diff(median),
                None => "-".to_string(),
            };

            let row = Row::new(
                [
                    String::new().into(),      // model (blank)
                    String::new().into(),      // bench (blank)
                    label.into(),              // AGENTS.md
                    pct_pass.into(),           // % pass
                    passed_str.into(),         // passed
                    group.totals.prompt_tokens.to_string().into(),
                    group.totals.completion_tokens.to_string().into(),
                    nreason_str.into(),        // reason
                    cost_str.into(),           // total cost
                    cost_diff_str.into(),      // % cost Δ med
                ],
                widths,
            );
            println!("{}", row.render());
        }
    }
}
