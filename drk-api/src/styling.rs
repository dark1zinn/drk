//! Terminal Styling Utilities
//! 
//! This module provides consistent styling functions for the drk CLI.
//! All styling goes through here to ensure visual consistency.

pub use console;

use console::{Style, style};

/// Style for primary/important text (cyan, bold)
/// 
/// # Example
/// ```
/// let text = style_primary("Hello");
/// println!("{}", text);
/// ```
pub fn style_primary(text: &str) -> console::StyledObject<&str> {
    style(text).cyan().bold()
}

/// Style for success messages (green)
pub fn style_success(text: &str) -> console::StyledObject<&str> {
    style(text).green()
}

/// Style for warnings (yellow)
pub fn style_warning(text: &str) -> console::StyledObject<&str> {
    style(text).yellow()
}

/// Style for errors (red, bold)
pub fn style_error(text: &str) -> console::StyledObject<&str> {
    style(text).red().bold()
}

/// Style for dimmed/secondary text (dim white)
pub fn style_dim(text: &str) -> console::StyledObject<&str> {
    style(text).dim()
}

/// Create a custom style
/// 
/// # Example
/// ```
/// let my_style = create_style().magenta().italic();
/// ```
pub fn create_style() -> Style {
    Style::new()
}

// --- Advanced: Icon Support ---

/// Cross-platform checkmark icon
pub fn icon_success() -> &'static str {
    if console::Term::stdout().features().is_attended() {
        "✓"  // Unicode checkmark
    } else {
        "[OK]"  // Fallback for non-terminal output
    }
}

/// Cross-platform error icon  
pub fn icon_error() -> &'static str {
    if console::Term::stdout().features().is_attended() {
        "✗"  // Unicode X
    } else {
        "[ERROR]"
    }
}

/// Cross-platform warning icon
pub fn icon_warning() -> &'static str {
    if console::Term::stdout().features().is_attended() {
        "⚠"  // Unicode warning
    } else {
        "[WARN]"
    }
}

/// Cross-platform info icon
pub fn icon_info() -> &'static str {
    if console::Term::stdout().features().is_attended() {
        "ℹ"  // Unicode info
    } else {
        "[INFO]"
    }
}
