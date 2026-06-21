use crate::token::Token;
use crate::style::{Style, StyleAttributes};
use crate::formatter::Formatter;
use std::collections::HashMap;
use std::io::Write;

/// IRC color index mapping (0-15).
const IRC_COLOR_MAP: &[(&str, u8)] = &[
    ("white", 0),
    ("black", 1),
    ("blue", 2),
    ("brightgreen", 3),
    ("brightred", 4),
    ("yellow", 5),
    ("magenta", 6),
    ("orange", 7),
    ("green", 7),        // compat w/ ansi
    ("brightyellow", 8),
    ("lightgreen", 9),
    ("brightcyan", 9),   // compat w/ ansi
    ("cyan", 10),
    ("lightblue", 11),
    ("red", 11),         // compat w/ ansi
    ("brightblue", 12),
    ("brightmagenta", 13),
    ("brightblack", 14),
    ("gray", 15),
];

/// Default IRC color scheme: (light_bg, dark_bg) per token.
const IRC_COLORS: &[(Token, (&str, &str))] = &[
    (Token::TOKEN, ("", "")),
    (Token::WHITESPACE, ("gray", "brightblack")),
    (Token::COMMENT, ("gray", "brightblack")),
    (Token::COMMENT_PREPROC, ("cyan", "brightcyan")),
    (Token::KEYWORD, ("blue", "brightblue")),
    (Token::KEYWORD_TYPE, ("cyan", "brightcyan")),
    (Token::OPERATOR_WORD, ("magenta", "brightcyan")),
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
    (Token::GENERIC_ERROR, ("brightred", "brightred")),
    (Token::ERROR, ("_brightred_", "_brightred_")),
];

/// Look up IRC color index by name.
fn irc_color_index(name: &str) -> Option<u8> {
    IRC_COLOR_MAP.iter().find(|(n, _)| *n == name).map(|(_, idx)| *idx)
}

/// Format a color spec string into IRC control codes.
/// Supports prefixes: `_` for italic, `*` for bold.
fn irc_format(color: &str) -> (String, String) {
    if color.is_empty() {
        return (String::new(), String::new());
    }

    let mut add = String::new();
    let mut sub = String::new();

    let italic = color.contains('_');
    let bold = color.contains('*');
    let color = color.trim_matches(|c| c == '_' || c == '*');
    let color = color.replace('_', "").replace('*', "");

    if italic {
        add.push('\x1D');
        sub.insert(0, '\x1D');  // prepend
    }
    if bold {
        add.push('\x02');
        sub.insert(0, '\x02');  // prepend
    }
    if let Some(idx) = irc_color_index(&color) {
        add.push('\x03');
        add.push_str(&format!("{:02}", idx));
        sub.insert(0, '\x03');  // prepend
    }

    (add, sub)
}

/// IRC formatter with color sequences.
pub struct IRCFormatter {
    colorscheme: HashMap<Token, (&'static str, &'static str)>,
    darkbg: bool,
    _linenos: bool,
}

impl IRCFormatter {
    pub fn new() -> Self {
        Self {
            colorscheme: IRC_COLORS.iter().cloned().collect(),
            darkbg: false,
            _linenos: false,
        }
    }

    pub fn dark() -> Self {
        Self {
            colorscheme: IRC_COLORS.iter().cloned().collect(),
            darkbg: true,
            _linenos: false,
        }
    }

    /// Look up style for a token, walking the hierarchy.
    fn get_style(&self, token: Token) -> Option<(&'static str, &'static str)> {
        let mut current = token;
        loop {
            if let Some(style) = self.colorscheme.get(&current) {
                return Some(*style);
            }
            let path = current.path;
            if path.len() <= 1 {
                break;
            }
            current = Token { path: &path[..path.len() - 1] };
        }
        None
    }
}

impl Default for IRCFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for IRCFormatter {
    fn name(&self) -> &str { "IRC" }
    fn extension(&self) -> &str { "" }
    fn mimetype(&self) -> &str { "text/plain" }

    fn format(&self, tokens: &[(Token, String)], outfile: &mut dyn Write) -> std::io::Result<()> {
        for (ttype, value) in tokens {
            let style = self.get_style(*ttype);
            if let Some((light, dark)) = style {
                let color = if self.darkbg { dark } else { light };
                let (add, sub) = irc_format(color);
                let parts: Vec<&str> = value.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if !part.is_empty() {
                        outfile.write_all(add.as_bytes())?;
                        outfile.write_all(part.as_bytes())?;
                        outfile.write_all(sub.as_bytes())?;
                    }
                    if i < parts.len() - 1 {
                        outfile.write_all(b"\n")?;
                    }
                }
            } else {
                outfile.write_all(value.as_bytes())?;
            }
        }
        Ok(())
    }
}

/// BBCode formatter.
///
/// Uses style attributes to generate BBCode tags for color, bold, italic, and underline.
pub struct BBCodeFormatter {
    styles: HashMap<Token, (String, String)>, // (open_tags, close_tags)
    codetag: bool,
    monofont: bool,
}

impl BBCodeFormatter {
    pub fn new(style: Option<Style>) -> Self {
        let style = style.unwrap_or_else(|| Style::new("default"));
        let mut formatter = Self {
            styles: HashMap::new(),
            codetag: false,
            monofont: false,
        };
        formatter.build_styles(&style);
        formatter
    }

    pub fn with_codetag(mut self) -> Self {
        self.codetag = true;
        self
    }

    pub fn with_monofont(mut self) -> Self {
        self.monofont = true;
        self
    }

    fn build_styles(&mut self, style: &Style) {
        for (ttype, attrs) in style.iter_styles() {
            let mut start = String::new();
            let mut end = String::new();

            if let Some(ref color) = attrs.color {
                start.push_str(&format!("[color={}]", color));
                end.insert_str(0, "[/color]");
            }
            if attrs.bold.unwrap_or(false) {
                start.push_str("[b]");
                end.insert_str(0, "[/b]");
            }
            if attrs.italic.unwrap_or(false) {
                start.push_str("[i]");
                end.insert_str(0, "[/i]");
            }
            if attrs.underline.unwrap_or(false) {
                start.push_str("[u]");
                end.insert_str(0, "[/u]");
            }

            self.styles.insert(ttype, (start, end));
        }
    }

    /// Look up style for a token, walking the hierarchy.
    fn get_style(&self, token: Token) -> Option<&(String, String)> {
        let mut current = token;
        loop {
            if let Some(style) = self.styles.get(&current) {
                return Some(style);
            }
            let path = current.path;
            if path.len() <= 1 {
                break;
            }
            current = Token { path: &path[..path.len() - 1] };
        }
        None
    }
}

impl Default for BBCodeFormatter {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Formatter for BBCodeFormatter {
    fn name(&self) -> &str { "BBCode" }
    fn extension(&self) -> &str { "bb" }
    fn mimetype(&self) -> &str { "text/plain" }

    fn format(&self, tokens: &[(Token, String)], outfile: &mut dyn Write) -> std::io::Result<()> {
        if self.codetag {
            outfile.write_all(b"[code]")?;
        }
        if self.monofont {
            outfile.write_all(b"[font=monospace]")?;
        }

        let mut lastval = String::new();
        let mut lasttype: Option<Token> = None;

        for (ttype, value) in tokens {
            let current = *ttype;
            if self.styles.contains_key(&current) {
                if lasttype == Some(current) {
                    lastval.push_str(value);
                } else {
                    if let Some(lt) = lasttype {
                        if let Some((start, end)) = self.styles.get(&lt) {
                            outfile.write_all(start.as_bytes())?;
                            outfile.write_all(lastval.as_bytes())?;
                            outfile.write_all(end.as_bytes())?;
                        }
                    }
                    lastval = value.to_string();
                    lasttype = Some(current);
                }
            } else {
                // Unstyled token — flush last and write raw
                if let Some(lt) = lasttype {
                    if let Some((start, end)) = self.styles.get(&lt) {
                        outfile.write_all(start.as_bytes())?;
                        outfile.write_all(lastval.as_bytes())?;
                        outfile.write_all(end.as_bytes())?;
                    }
                }
                outfile.write_all(value.as_bytes())?;
                lastval.clear();
                lasttype = None;
            }
        }

        // Flush remaining
        if let Some(lt) = lasttype {
            if let Some((start, end)) = self.styles.get(&lt) {
                outfile.write_all(start.as_bytes())?;
                outfile.write_all(lastval.as_bytes())?;
                outfile.write_all(end.as_bytes())?;
            }
        }

        if self.monofont {
            outfile.write_all(b"[/font]")?;
        }
        if self.codetag {
            outfile.write_all(b"[/code]")?;
        }
        if self.codetag || self.monofont {
            outfile.write_all(b"\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tokens() -> Vec<(Token, String)> {
        vec![
            (Token::KEYWORD, "def".to_string()),
            (Token::NAME, " hello".to_string()),
            (Token::STRING, "\"world\"".to_string()),
        ]
    }

    #[test]
    fn test_irc_color_index() {
        assert_eq!(irc_color_index("blue"), Some(2));
        assert_eq!(irc_color_index("yellow"), Some(5));
        assert_eq!(irc_color_index("gray"), Some(15));
        assert_eq!(irc_color_index("unknown"), None);
    }

    #[test]
    fn test_irc_format_plain() {
        let (add, sub) = irc_format("blue");
        assert_eq!(add, "\x0302");
        assert_eq!(sub, "\x03");
    }

    #[test]
    fn test_irc_format_bold() {
        let (add, sub) = irc_format("*blue*");
        println!("bold add = {:?}, sub = {:?}", add, sub);
        assert_eq!(add, "\x02\x0302");
        assert_eq!(sub, "\x03\x02");
    }

    #[test]
    fn test_irc_format_italic() {
        let (add, sub) = irc_format("_cyan_");
        println!("italic add = {:?}, sub = {:?}", add, sub);
        assert_eq!(add, "\x1D\x0310");
        assert_eq!(sub, "\x03\x1D");
    }

    #[test]
    fn test_irc_format_empty() {
        let (add, sub) = irc_format("");
        assert_eq!(add, "");
        assert_eq!(sub, "");
    }

    #[test]
    fn test_irc_formatter_light() {
        let formatter = IRCFormatter::new();
        let tokens = sample_tokens();
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("def"));
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
        assert!(result.contains("\x03"));
    }

    #[test]
    fn test_irc_formatter_dark() {
        let formatter = IRCFormatter::dark();
        let tokens = sample_tokens();
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("def"));
        assert!(result.contains("\x03"));
    }

    #[test]
    fn test_bbcode_formatter() {
        let mut style = Style::new("test");
        style.styles.insert(Token::KEYWORD, StyleAttributes::from_css_string("color:#0000ff;bold:true"));
        style.styles.insert(Token::NAME, StyleAttributes::from_css_string("color:#000000"));
        style.styles.insert(Token::STRING, StyleAttributes::from_css_string("color:#ba2121"));
        let formatter = BBCodeFormatter::new(Some(style));
        let tokens = sample_tokens();
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        println!("BBCode output: {:?}", result);
        assert!(result.contains("def"));
        assert!(result.contains("hello"));
        // Check that BBCode tags are present
        assert!(result.contains("[color=#0000ff]"), "Missing keyword color");
        assert!(result.contains("[/color]"), "Missing close color tag");
        assert!(result.contains("[b]"), "Missing bold tag");
        assert!(result.contains("[/b]"), "Missing close bold tag");
    }

    #[test]
    fn test_bbcode_with_codetag() {
        let formatter = BBCodeFormatter::new(None).with_codetag();
        let tokens = sample_tokens();
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.starts_with("[code]"));
        assert!(result.ends_with("[/code]\n"));
    }

    #[test]
    fn test_bbcode_with_monofont() {
        let formatter = BBCodeFormatter::new(None).with_monofont();
        let tokens = sample_tokens();
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("[font=monospace]"));
        assert!(result.contains("[/font]"));
    }
}
