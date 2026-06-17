use std::collections::HashMap;
use crate::token::Token;
use crate::style::{Style, StyleAttributes};
use super::{Formatter, escape_html, token_to_class};

/// Create a default style with basic colors for inline mode.
fn default_style() -> Style {
    let mut style = Style::new("default");
    style.styles.insert(Token::KEYWORD, StyleAttributes::from_css_string("color:#008000;bold:true"));
    style.styles.insert(Token::KEYWORD_DECLARATION, StyleAttributes::from_css_string("color:#008000;bold:true"));
    style.styles.insert(Token::KEYWORD_NAMESPACE, StyleAttributes::from_css_string("color:#008000;bold:true"));
    style.styles.insert(Token::NAME, StyleAttributes::from_css_string("color:#000000"));
    style.styles.insert(Token::NAME_FUNCTION, StyleAttributes::from_css_string("color:#0000FF"));
    style.styles.insert(Token::NAME_CLASS, StyleAttributes::from_css_string("color:#0000FF;bold:true"));
    style.styles.insert(Token::NAME_BUILTIN, StyleAttributes::from_css_string("color:#007020"));
    style.styles.insert(Token::STRING, StyleAttributes::from_css_string("color:#BA2121"));
    style.styles.insert(Token::NUMBER, StyleAttributes::from_css_string("color:#666666"));
    style.styles.insert(Token::COMMENT, StyleAttributes::from_css_string("color:#408080;font-style:italic"));
    style.styles.insert(Token::OPERATOR, StyleAttributes::from_css_string("color:#666666"));
    style.styles.insert(Token::PUNCTUATION, StyleAttributes::from_css_string("color:#000000"));
    style.styles.insert(Token::WHITESPACE, StyleAttributes::from_css_string("color:#BBBBBB"));
    style
}

/// HTML formatter options.
#[derive(Debug, Clone)]
pub struct HtmlFormatterOptions {
    pub cssclass: String,
    pub classname: String,
    pub noclasses: bool,
    pub classes: HashMap<Token, String>,
    pub style: Option<String>,
    pub stripnl: bool,
    pub linenos: bool,
    pub anchorlinenos: bool,
    pub lineno_start: usize,
    pub linenospecial: Option<usize>,
    pub linespans: usize,
    pub lineanchors: String,
    pub wrapcode: bool,
    pub full: bool,
    pub cssstyles: HashMap<String, String>,
    pub title: String,
    pub glslslang: String,
    pub unicodeescape: bool,
    pub webpath: String,
    pub tabsize: usize,
}

impl Default for HtmlFormatterOptions {
    fn default() -> Self {
        HtmlFormatterOptions {
            cssclass: "highlight".to_string(),
            classname: "highlight".to_string(),
            noclasses: false,
            classes: HashMap::new(),
            style: None,
            stripnl: true,
            linenos: false,
            anchorlinenos: false,
            lineno_start: 1,
            linenospecial: None,
            linespans: 0,
            lineanchors: String::new(),
            wrapcode: false,
            full: false,
            cssstyles: HashMap::new(),
            title: String::new(),
            glslslang: String::new(),
            unicodeescape: false,
            webpath: String::new(),
            tabsize: 8,
        }
    }
}

/// HTML formatter — produces HTML with CSS classes or inline styles.
pub struct HtmlFormatter {
    pub options: HtmlFormatterOptions,
    pub style: Option<Style>,
}

impl HtmlFormatter {
    pub fn new(options: Option<HtmlFormatterOptions>) -> Self {
        let mut opts = options.unwrap_or_default();
        let style = if opts.noclasses {
            // For noclasses mode, use a default style with basic colors
            Some(default_style())
        } else {
            None
        };
        HtmlFormatter {
            options: opts,
            style,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Generate CSS for the style.
    pub fn generate_css(&self) -> String {
        let cssclass = &self.options.classname;
        let mut css = Vec::new();

        if let Some(ref style) = self.style {
            if self.options.noclasses {
                // Inline styles — generate .<cssclass> .<token-class> rules
                for (token, attrs) in style.iter_styles() {
                    let class = token_to_class(token);
                    if class.is_empty() {
                        continue;
                    }
                    let mut declarations = Vec::new();
                    if let Some(ref color) = attrs.color {
                        declarations.push(format!("color: {}", color));
                    }
                    if let Some(ref bg) = attrs.bg {
                        declarations.push(format!("background-color: {}", bg));
                    }
                    if attrs.bold == Some(true) {
                        declarations.push("font-weight: bold".to_string());
                    }
                    if attrs.italic == Some(true) {
                        declarations.push("font-style: italic".to_string());
                    }
                    if attrs.underline == Some(true) {
                        declarations.push("text-decoration: underline".to_string());
                    }
                    if !declarations.is_empty() {
                        css.push(format!(".{} .{} {{ {} }}", cssclass, class, declarations.join("; ")));
                    }
                }
            } else {
                // Class-based — generate style definitions
                if let Some(ref default_attrs) = style.styles.get(&Token::TOKEN) {
                    let mut declarations = Vec::new();
                    if let Some(ref color) = default_attrs.color {
                        declarations.push(format!("color: {}", color));
                    }
                    if let Some(ref bg) = default_attrs.bg {
                        declarations.push(format!("background-color: {}", bg));
                    }
                    if !declarations.is_empty() {
                        css.push(format!(".{} pre {{ {} }}", cssclass, declarations.join("; ")));
                    }
                }
            }
        }

        css.join("\n")
    }

    /// Format tokens with inline styles (noclasses mode).
    fn format_inline(&self, tokens: &[(Token, String)], outfile: &mut dyn std::io::Write) -> std::io::Result<()> {
        let cssclass = &self.options.classname;

        write!(outfile, "<div class=\"{}\"><pre tabindex=\"0\">", cssclass)?;

        let mut current_line = 1;
        for (token, text) in tokens {
            // Handle newlines
            let lines: Vec<&str> = text.split('\n').collect();
            for (i, line_part) in lines.iter().enumerate() {
                if i > 0 {
                    if self.options.linenos {
                        write!(outfile, "</span>\n<span class=\"{}\">", "ln")?;
                        write!(outfile, "<span class=\"{}\">{}</span>", "lnt", current_line)?;
                        write!(outfile, "<span class=\"{}\">", "w")?;
                    }
                    current_line += 1;
                }

                if !line_part.is_empty() {
                    let class = token_to_class(*token);
                    let escaped = escape_html(line_part);

                    if self.options.noclasses {
                        // Inline style
                        let style_str = self.get_inline_style(*token);
                        if !style_str.is_empty() {
                            write!(outfile, "<span style=\"{}\">{}</span>", style_str, escaped)?;
                        } else {
                            write!(outfile, "{}", escaped)?;
                        }
                    } else {
                        // Class-based
                        if !class.is_empty() {
                            write!(outfile, "<span class=\"{}\">{}</span>", class, escaped)?;
                        } else {
                            write!(outfile, "{}", escaped)?;
                        }
                    }
                }
            }
        }

        write!(outfile, "</pre>\n</div>")?;
        Ok(())
    }

    /// Get inline CSS style for a token.
    fn get_inline_style(&self, token: Token) -> String {
        let mut declarations = Vec::new();

        if let Some(ref style) = self.style {
            let attrs = style.style_for_token(token);
            if let Some(ref color) = attrs.color {
                declarations.push(format!("color: {}", color));
            }
            if let Some(ref bg) = attrs.bg {
                declarations.push(format!("background-color: {}", bg));
            }
            if attrs.bold == Some(true) {
                declarations.push("font-weight: bold".to_string());
            }
            if attrs.italic == Some(true) {
                declarations.push("font-style: italic".to_string());
            }
            if attrs.underline == Some(true) {
                declarations.push("text-decoration: underline".to_string());
            }
        }

        declarations.join("; ")
    }

    /// Format tokens with class names.
    fn format_classes(&self, tokens: &[(Token, String)], outfile: &mut dyn std::io::Write) -> std::io::Result<()> {
        let cssclass = &self.options.classname;

        write!(outfile, "<div class=\"{}\"><pre tabindex=\"0\">", cssclass)?;

        let mut current_line = 1;
        for (token, text) in tokens {
            let lines: Vec<&str> = text.split('\n').collect();
            for (i, line_part) in lines.iter().enumerate() {
                if i > 0 {
                    if self.options.linenos {
                        write!(outfile, "</span>\n<span class=\"{}\">", "ln")?;
                        write!(outfile, "<span class=\"{}\">{}</span>", "lnt", current_line)?;
                        write!(outfile, "<span class=\"{}\">", "w")?;
                    }
                    current_line += 1;
                }

                if !line_part.is_empty() {
                    let class = token_to_class(*token);
                    let escaped = escape_html(line_part);

                    if !class.is_empty() {
                        write!(outfile, "<span class=\"{}\">{}</span>", class, escaped)?;
                    } else {
                        write!(outfile, "{}", escaped)?;
                    }
                }
            }
        }

        write!(outfile, "</pre>\n</div>")?;
        Ok(())
    }
}

impl Formatter for HtmlFormatter {
    fn name(&self) -> &str {
        "HTML"
    }

    fn extension(&self) -> &str {
        "html"
    }

    fn mimetype(&self) -> &str {
        "text/html"
    }

    fn format(
        &self,
        tokens: &[(Token, String)],
        outfile: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        if self.options.noclasses {
            self.format_inline(tokens, outfile)
        } else {
            self.format_classes(tokens, outfile)
        }
    }

    fn get_style_defs(&self, _arg: Option<&str>) -> String {
        self.generate_css()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_formatter_basic() {
        let formatter = HtmlFormatter::new(None);
        let tokens = vec![
            (Token::KEYWORD, "if".to_string()),
            (Token::WHITESPACE, " ".to_string()),
            (Token::NAME, "x".to_string()),
        ];
        let mut output = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let html = String::from_utf8(output).unwrap();
        assert!(html.contains("class=\"k\""));
        assert!(html.contains("class=\"n\""));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>test&\"</div>"), "&lt;div&gt;test&amp;&quot;&lt;/div&gt;");
    }
}
