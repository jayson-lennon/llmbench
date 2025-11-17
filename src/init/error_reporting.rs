use error_stack::{Report, fmt::ColorMode};

use crate::error::{ErrContext, Suggestion};

pub fn init_error_stack() {
    Report::set_color_mode(ColorMode::Color);
    ErrContext::install_hook();
    Suggestion::install_hook();
}
