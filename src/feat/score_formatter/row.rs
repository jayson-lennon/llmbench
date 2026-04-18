use std::borrow::Cow;

use ansi_width::ansi_width;

use super::NCOLS;

/// Holds the computed widths for each column.
#[derive(Debug, Clone)]
pub(super) struct ColWidths {
    pub(super) bench: usize,
    pub(super) agent: usize,
    pub(super) model: usize,
    pub(super) result: usize,
    pub(super) passed: usize,
    pub(super) in_tokens: usize,
    pub(super) out_tokens: usize,
    pub(super) reason: usize,
    pub(super) cost: usize,
    pub(super) cost_diff: usize,
}

impl ColWidths {
    pub(super) fn as_array(&self) -> [usize; NCOLS] {
        [
            self.bench,
            self.agent,
            self.model,
            self.result,
            self.passed,
            self.in_tokens,
            self.out_tokens,
            self.reason,
            self.cost,
            self.cost_diff,
        ]
    }
}

/// A single row of pre-padded cell values.
pub(super) struct Row([Cow<'static, str>; NCOLS]);

impl Row {
    /// Build a row from raw values + column widths.
    pub(super) fn new(cols: [Cow<'_, str>; NCOLS], widths: &ColWidths) -> Self {
        let w = widths.as_array();
        let cells: Vec<Cow<'static, str>> = cols
            .into_iter()
            .zip(w)
            .map(|(val, width)| pad_str(&val, width, ' ').into_owned().into())
            .collect();
        Self(cells.try_into().unwrap_or_else(|v: Vec<_>| {
            panic!("expected {NCOLS} cells, got {}", v.len())
        }))
    }

    /// Header labels.
    pub(super) fn header(w: &ColWidths) -> Self {
        Self::new(
            [
                "bench".into(),
                "agent".into(),
                "model".into(),
                "result".into(),
                "passed".into(),
                "in".into(),
                "out".into(),
                "reason".into(),
                "cost ($USD)    ".into(),
                "% cost diff  ".into(),
            ],
            w,
        )
    }

    /// Summary header (% pass replaces result).
    pub(super) fn summary_header(w: &ColWidths) -> Self {
        let mut row = Self::header(w);
        row.0[3] = pad_str("% pass", w.result, ' ').into_owned().into();
        row
    }

    /// Separator line of dashes.
    pub(super) fn separator(w: &ColWidths) -> Self {
        let w = w.as_array();
        let cells: Vec<Cow<'static, str>> = w
            .iter()
            .map(|&width| pad_str("", width, '-').into_owned().into())
            .collect();
        Self(cells.try_into().unwrap())
    }

    /// Render the row with ` | ` dividers.
    pub(super) fn render(&self, sep: &str) -> String {
        let mut out = String::from(sep);
        for cell in &self.0 {
            out.push_str(cell);
            out.push_str(sep);
        }
        out
    }
}

pub(super) fn pad_str(input: &str, amount: usize, ch: char) -> Cow<'_, str> {
    let visual_len = ansi_width(input);
    if visual_len < amount {
        let diff = amount - visual_len;
        let mut out = String::with_capacity(input.len() + diff);
        out.push_str(input);
        out.extend((0..diff).map(|_| ch));
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}
