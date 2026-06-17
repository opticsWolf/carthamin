use crate::token::Token;
use crate::formatter::Formatter;
use std::io::Write;
use std::collections::HashMap;

/// ANSI escape code prefix.
const ESC: &str = "\x1b[";

/// ANSI color codes for terminal output.
const RESET: &str = "\x1b[39;49;00m";
const BOLD: &str = "\x1b[01m";
const UNDERLINE: &str = "\x1b[04m";
const BLINK: &str = "\x1b[05m";

/// Map color names to ANSI escape sequences.
const COLOR_CODES: &[(&str, &str)] = &[
    ("", ""),
    ("reset", RESET),
    ("bold", BOLD),
    ("underline", UNDERLINE),
    ("blink", BLINK),
    ("black", "\x1b[30m"),
    ("red", "\x1b[31m"),
    ("green", "\x1b[32m"),
    ("yellow", "\x1b[33m"),
    ("blue", "\x1b[34m"),
    ("magenta", "\x1b[35m"),
    ("cyan", "\x1b[36m"),
    ("gray", "\x1b[90m"),
    ("brightblack", "\x1b[90m"),
    ("brightred", "\x1b[91m"),
    ("brightgreen", "\x1b[92m"),
    ("brightyellow", "\x1b[93m"),
    ("brightblue", "\x1b[94m"),
    ("brightmagenta", "\x1b[95m"),
    ("brightcyan", "\x1b[96m"),
    ("white", "\x1b[97m"),
];

/// Default terminal color scheme mapping tokens to (light_bg, dark_bg) colors.
pub const TERMINAL_COLORS: &[(Token, (&str, &str))] = &[
    (Token::TOKEN, ("", "")),
    (Token::WHITESPACE, ("gray", "brightblack")),
    (Token::COMMENT, ("gray", "brightblack")),
    (Token::COMMENT_PREPROC, ("cyan", "brightcyan")),
    (Token::KEYWORD, ("blue", "brightblue")),
    (Token::KEYWORD_TYPE, ("cyan", "brightcyan")),
    (Token::OPERATOR_WORD, ("magenta", "brightmagenta")),
    (Token::NAME_BUILTIN, ("cyan", "brightcyan")),
    (Token::NAME_FUNCTION, ("green", "brightgreen")),
    (Token::NAME_NAMESPACE, ("_cyan_", "_brightcyan_")),
    (Token::NAME_CLASS, ("_green_", "_brightgreen_")),
    (Token::NAME_EXCEPTION, ("cyan", "brightcyan")),
    (Token::NAME_DECORATOR, ("brightblack", "gray")),
    (Token::NAME_VARIABLE, ("red", "brightred")),
    (Token::NAME_CONSTANT, ("red", "brightred")),
    (Token::NAME_ATTRIBUTE, ("cyan", "brightcyan")),
    (Token::NAME_TAG, ("brightblue", "brightblue")),
    (Token::STRING, ("yellow", "yellow")),
    (Token::NUMBER, ("blue", "brightblue")),
    (Token::GENERIC_DELETED, ("brightred", "brightred")),
    (Token::GENERIC_INSERTED, ("green", "brightgreen")),
    (Token::GENERIC_HEADING, ("**", "**")),
    (Token::GENERIC_SUBHEADING, ("*magenta*", "*brightmagenta*")),
    (Token::GENERIC_PROMPT, ("**", "**")),
    (Token::GENERIC_ERROR, ("brightred", "brightred")),
    (Token::ERROR, ("_brightred_", "_brightred_")),
];

/// ANSI format a string with color attributes.
/// Supports formats: "color", "*color*", "_color_", "+color+"
pub fn ansiformat(attr: &str, text: &str) -> String {
    let mut result = String::new();
    let mut current = attr;

    if current.starts_with('+') && current.ends_with('+') && current.len() > 2 {
        result.push_str(BLINK);
        current = &current[1..current.len()-1];
    }
    if current.starts_with('*') && current.ends_with('*') && current.len() > 2 {
        result.push_str(BOLD);
        current = &current[1..current.len()-1];
    }
    if current.starts_with('_') && current.ends_with('_') && current.len() > 2 {
        result.push_str(UNDERLINE);
        current = &current[1..current.len()-1];
    }

    // Look up the color code
    for (name, code) in COLOR_CODES {
        if *name == current {
            result.push_str(code);
            break;
        }
    }

    result.push_str(text);
    result.push_str(RESET);
    result
}

/// Terminal formatter with ANSI color sequences.
pub struct TerminalFormatter {
    pub darkbg: bool,
    pub colorscheme: HashMap<Token, (&'static str, &'static str)>,
    pub linenos: bool,
}

impl TerminalFormatter {
    pub fn new(options: Option<&HashMap<String, String>>) -> Self {
        let empty = HashMap::new();
        let opts = options.unwrap_or(&empty);
        let darkbg = opts.get("bg").map(|s| s == "dark").unwrap_or(false);
        let linenos = opts.get("linenos").map(|s| s == "True" || s == "1").unwrap_or(false);

        let colorscheme: HashMap<Token, (&'static str, &'static str)> = TERMINAL_COLORS.iter()
            .map(|(t, c)| (*t, *c))
            .collect();

        TerminalFormatter {
            darkbg,
            colorscheme,
            linenos,
        }
    }

    /// Get the color for a token type, walking up the hierarchy.
    fn get_color(&self, ttype: &Token) -> Option<&'static str> {
        let mut current = *ttype;
        for _ in 0..current.path.len() + 1 {
            if let Some(&(light, dark)) = self.colorscheme.get(&current) {
                return if self.darkbg { Some(dark) } else { Some(light) };
            }
            // Walk up the hierarchy
            if current.path.is_empty() {
                break;
            }
            // Get parent by looking up in ALL_TOKENS
            let parent_path = &current.path[..current.path.len()-1];
            current = crate::token::ALL_TOKENS.iter()
                .find(|(_, p)| *p == parent_path)
                .map(|(t, _)| *t)
                .unwrap_or(Token::TOKEN);
        }
        // Fallback to base Token
        if let Some(&(light, dark)) = self.colorscheme.get(&Token::TOKEN) {
            return if self.darkbg { Some(dark) } else { Some(light) };
        }
        None
    }
}

impl Formatter for TerminalFormatter {
    fn name(&self) -> &str { "Terminal" }
    fn extension(&self) -> &str { "" }
    fn mimetype(&self) -> &str { "text/plain" }

    fn format(&self, tokens: &[(Token, String)], outfile: &mut dyn Write) -> std::io::Result<()> {
        let mut lineno: usize = 0;

        if self.linenos {
            writeln!(outfile, "{:04}: ", lineno + 1).ok();
            lineno = 1;
        }

        for (ttype, text) in tokens {
            let color = self.get_color(ttype);

            for line in text.split_inclusive('\n') {
                let line_trimmed = line.trim_end_matches('\n');
                if let Some(c) = color {
                    if !c.is_empty() {
                        outfile.write_all(ansiformat(c, line_trimmed).as_bytes())?;
                    } else {
                        outfile.write_all(line_trimmed.as_bytes())?;
                    }
                } else {
                    outfile.write_all(line_trimmed.as_bytes())?;
                }

                if line.ends_with('\n') {
                    if self.linenos {
                        lineno += 1;
                        writeln!(outfile, "{:04}: ", lineno + 1).ok();
                    } else {
                        writeln!(outfile)?;
                    }
                }
            }
        }

        if self.linenos {
            writeln!(outfile)?;
        }

        Ok(())
    }

    fn get_style_defs(&self, _arg: Option<&str>) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansiformat_basic() {
        let result = ansiformat("red", "hello");
        assert!(result.contains("\x1b[31m"));
        assert!(result.contains("hello"));
        assert!(result.contains(RESET));
    }

    #[test]
    fn test_ansiformat_bold() {
        let result = ansiformat("*blue*", "hello");
        assert!(result.contains(BOLD));
        assert!(result.contains("\x1b[34m"));
    }

    #[test]
    fn test_terminal_formatter() {
        let mut formatter = TerminalFormatter::new(None);
        let tokens = vec![
            (Token::KEYWORD, "if".to_string()),
            (Token::TEXT, " ".to_string()),
            (Token::NAME, "x".to_string()),
        ];

        let mut output = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("x"));
    }
}
