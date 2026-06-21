pub mod html;
pub mod terminal;
pub mod terminal256;
pub mod other;
pub mod irc_bbcode;

use crate::token::Token;

/// Base trait for all formatters.
pub trait Formatter {
    /// Name of the formatter.
    fn name(&self) -> &str;

    /// Default file extension for output.
    fn extension(&self) -> &str;

    /// MIME type of output.
    fn mimetype(&self) -> &str;

    /// Format a token stream to the output writer.
    fn format(
        &self,
        tokens: &[(Token, String)],
        outfile: &mut dyn std::io::Write,
    ) -> std::io::Result<()>;

    /// Get style definitions (CSS, LaTeX commands, etc.).
    fn get_style_defs(&self, _arg: Option<&str>) -> String {
        String::new()
    }
}

/// Escape XML/HTML special characters.
pub fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            c => result.push(c),
        }
    }
    result
}

/// Get the CSS class name for a token type.
pub fn token_to_class(token: Token) -> &'static str {
    use crate::token::STANDARD_TYPES;
    STANDARD_TYPES.get(&token).copied().unwrap_or("")
}
