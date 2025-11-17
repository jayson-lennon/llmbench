use derive_more::Display;
use owo_colors::OwoColorize;

use error_stack::Report;
use error_stack::fmt::ColorMode;

/// A suggestion that can be attached to an error report.
#[derive(Debug, Display)]
#[display("{_0}")]
pub struct Suggestion(pub &'static str);

impl Suggestion {
    /// Installs the reporting hook.
    pub fn install_hook() {
        Report::install_debug_hook::<Suggestion>(|Suggestion(value), context| {
            let body = format!("suggestion: {value}");
            match context.color_mode() {
                ColorMode::Color => context.push_body(body.cyan().to_string()),
                ColorMode::Emphasis => context.push_body(body.italic().to_string()),
                ColorMode::None => context.push_body(body),
            }
        });
    }
}

/// Attach error context to display when reporting an error
#[derive(Debug, Display)]
#[display("{_0}")]
pub struct ErrContext(pub String);
impl ErrContext {
    pub fn new<S>(context: S) -> Self
    where
        S: Into<String>,
    {
        Self(context.into())
    }
}

impl ErrContext {
    /// Installs the reporting hook.
    pub fn install_hook() {
        Report::install_debug_hook::<ErrContext>(|ErrContext(value), context| {
            let body = format!("context: {value}");
            match context.color_mode() {
                ColorMode::Color => context.push_body(body.yellow().to_string()),
                ColorMode::Emphasis => context.push_body(body.italic().to_string()),
                ColorMode::None => context.push_body(body),
            }
        });
    }
}
