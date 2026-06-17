use std::collections::HashMap;
use std::io::Write;
use crate::token::Token;
use crate::style::{Style, StyleAttributes};
use crate::formatter::Formatter;

/// XTerm 256-color palette (RGB values).
fn xterm_colors() -> [(u8, u8, u8); 256] {
    let mut colors = [(0u8, 0u8, 0u8); 256];

    // Colors 0-15: 16 basic colors
    colors[0] = (0x00, 0x00, 0x00);
    colors[1] = (0xcd, 0x00, 0x00);
    colors[2] = (0x00, 0xcd, 0x00);
    colors[3] = (0xcd, 0xcd, 0x00);
    colors[4] = (0x00, 0x00, 0xee);
    colors[5] = (0xcd, 0x00, 0xcd);
    colors[6] = (0x00, 0xcd, 0xcd);
    colors[7] = (0xe5, 0xe5, 0xe5);
    colors[8] = (0x7f, 0x7f, 0x7f);
    colors[9] = (0xff, 0x00, 0x00);
    colors[10] = (0x00, 0xff, 0x00);
    colors[11] = (0xff, 0xff, 0x00);
    colors[12] = (0x5c, 0x5c, 0xff);
    colors[13] = (0xff, 0x00, 0xff);
    colors[14] = (0x00, 0xff, 0xff);
    colors[15] = (0xff, 0xff, 0xff);

    // Colors 16-231: 6x6x6 color cube
    let values = [0x00u8, 0x5f, 0x87, 0xaf, 0xd7, 0xff];
    for i in 0..216 {
        let r = values[(i / 36) % 6];
        let g = values[(i / 6) % 6];
        let b = values[i % 6];
        colors[16 + i] = (r, g, b);
    }

    // Colors 232-255: grayscale
    for i in 0..24 {
        let v = (8 + i * 10) as u8;
        colors[232 + i] = (v, v, v);
    }

    colors
}

/// Find the closest XTerm 256-color index for an RGB color.
fn closest_color(r: u8, g: u8, b: u8, palette: &[(u8, u8, u8); 256]) -> u8 {
    let mut best_dist = u32::MAX;
    let mut best_idx = 0u8;

    for (i, &(pr, pg, pb)) in palette.iter().enumerate() {
        let dr = (r as i32 - pr as i32).unsigned_abs();
        let dg = (g as i32 - pg as i32).unsigned_abs();
        let db = (b as i32 - pb as i32).unsigned_abs();
        let dist = dr * dr + dg * dg + db * db;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i as u8;
        }
    }

    best_idx
}

/// Parse an RGB color string like "#RRGGBB" into (R, G, B).
fn parse_rgb(color: &str) -> Option<(u8, u8, u8)> {
    let color = color.trim_start_matches('#');
    if color.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&color[0..2], 16).ok()?;
    let g = u8::from_str_radix(&color[2..4], 16).ok()?;
    let b = u8::from_str_radix(&color[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Generate an ANSI 256-color escape sequence.
fn ansi_256_sequence(fg: Option<u8>, bg: Option<u8>, bold: bool, underline: bool, italic: bool) -> String {
    let mut attrs = Vec::new();

    if let Some(f) = fg {
        attrs.push("38".to_string());
        attrs.push("5".to_string());
        attrs.push(f.to_string());
    }
    if let Some(b) = bg {
        attrs.push("48".to_string());
        attrs.push("5".to_string());
        attrs.push(b.to_string());
    }
    if bold { attrs.push("01".to_string()); }
    if underline { attrs.push("04".to_string()); }
    if italic { attrs.push("03".to_string()); }

    if attrs.is_empty() {
        return String::new();
    }
    format!("\x1b[{}m", attrs.join(";"))
}

/// Generate an ANSI true-color escape sequence.
fn ansi_true_color_sequence(fg: Option<(u8, u8, u8)>, bg: Option<(u8, u8, u8)>, bold: bool, underline: bool, italic: bool) -> String {
    let mut attrs = Vec::new();

    if let Some((r, g, b)) = fg {
        attrs.push("38".to_string());
        attrs.push("2".to_string());
        attrs.push(r.to_string());
        attrs.push(g.to_string());
        attrs.push(b.to_string());
    }
    if let Some((r, g, b)) = bg {
        attrs.push("48".to_string());
        attrs.push("2".to_string());
        attrs.push(r.to_string());
        attrs.push(g.to_string());
        attrs.push(b.to_string());
    }
    if bold { attrs.push("01".to_string()); }
    if underline { attrs.push("04".to_string()); }
    if italic { attrs.push("03".to_string()); }

    if attrs.is_empty() {
        return String::new();
    }
    format!("\x1b[{}m", attrs.join(";"))
}

/// Generate a reset escape sequence.
fn reset_sequence(has_fg: bool, has_bg: bool, has_style: bool) -> String {
    let mut attrs = Vec::new();
    if has_fg { attrs.push("39".to_string()); }
    if has_bg { attrs.push("49".to_string()); }
    if has_style { attrs.push("00".to_string()); }

    if attrs.is_empty() {
        return String::new();
    }
    format!("\x1b[{}m", attrs.join(";"))
}

/// Terminal256Formatter: 256-color terminal formatter.
pub struct Terminal256Formatter {
    style_string: HashMap<Token, (String, String)>, // (on, off)
    _palette: [(u8, u8, u8); 256],
    best_match: HashMap<String, u8>,
    use_bold: bool,
    use_underline: bool,
    use_italic: bool,
    _linenos: bool,
}

impl Terminal256Formatter {
    pub fn new(style: Option<Style>) -> Self {
        let palette = xterm_colors();
        let style = style.unwrap_or_else(|| Style::new("default"));
        let mut formatter = Terminal256Formatter {
            style_string: HashMap::new(),
            _palette: palette,
            best_match: HashMap::new(),
            use_bold: true,
            use_underline: true,
            use_italic: true,
            _linenos: false,
        };
        formatter.setup_styles(&style, &palette);
        formatter
    }

    fn color_index(&mut self, color: &str, palette: &[(u8, u8, u8); 256]) -> u8 {
        if let Some(&idx) = self.best_match.get(color) {
            return idx;
        }

        // Try to parse as hex RGB
        if let Some((r, g, b)) = parse_rgb(color) {
            let idx = closest_color(r, g, b, palette);
            self.best_match.insert(color.to_string(), idx);
            return idx;
        }

        0
    }

    fn setup_styles(&mut self, style: &Style, palette: &[(u8, u8, u8); 256]) {
        for (token, attrs) in &style.styles {
            let fg = self.resolve_fg(attrs, palette);
            let bg = self.resolve_bg(attrs, palette);
            let bold = self.use_bold && attrs.bold.unwrap_or(false);
            let underline = self.use_underline && attrs.underline.unwrap_or(false);
            let italic = self.use_italic && attrs.italic.unwrap_or(false);

            let on = ansi_256_sequence(fg, bg, bold, underline, italic);
            let off = reset_sequence(fg.is_some(), bg.is_some(), bold || underline || italic);
            self.style_string.insert(*token, (on, off));
        }
    }

    fn resolve_fg(&mut self, attrs: &StyleAttributes, palette: &[(u8, u8, u8); 256]) -> Option<u8> {
        if let Some(ref color) = attrs.color {
            return Some(self.color_index(color, palette));
        }
        None
    }

    fn resolve_bg(&mut self, attrs: &StyleAttributes, palette: &[(u8, u8, u8); 256]) -> Option<u8> {
        if let Some(ref bg) = attrs.bg {
            return Some(self.color_index(bg, palette));
        }
        None
    }

    /// Look up style for a token, walking the hierarchy.
    fn get_style(&self, token: Token) -> Option<&(String, String)> {
        let mut current = token;
        loop {
            if let Some(style) = self.style_string.get(&current) {
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

    fn format_token(&self, ttype: Token, value: &str, outfile: &mut dyn Write) -> std::io::Result<()> {
        let style = self.get_style(ttype);
        let (on, off) = style.cloned().unwrap_or_default();

        let has_fg = !on.is_empty();

        let parts: Vec<&str> = value.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                if has_fg {
                    outfile.write_all(on.as_bytes())?;
                }
                outfile.write_all(part.as_bytes())?;
                if has_fg {
                    outfile.write_all(off.as_bytes())?;
                }
            }
            if i < parts.len() - 1 {
                outfile.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}

impl Formatter for Terminal256Formatter {
    fn name(&self) -> &str { "Terminal256" }
    fn extension(&self) -> &str { "" }
    fn mimetype(&self) -> &str { "text/plain" }

    fn format(&self, tokens: &[(Token, String)], outfile: &mut dyn Write) -> std::io::Result<()> {
        for (ttype, value) in tokens {
            if !value.is_empty() {
                self.format_token(*ttype, value, outfile)?;
            }
        }
        Ok(())
    }
}

/// TerminalTrueColorFormatter: True color (24-bit) terminal formatter.
pub struct TerminalTrueColorFormatter {
    style_string: HashMap<Token, (String, String)>,
    use_bold: bool,
    use_underline: bool,
    use_italic: bool,
    _linenos: bool,
}

impl TerminalTrueColorFormatter {
    pub fn new(style: Option<Style>) -> Self {
        let style = style.unwrap_or_else(|| Style::new("default"));
        let mut formatter = TerminalTrueColorFormatter {
            style_string: HashMap::new(),
            use_bold: true,
            use_underline: true,
            use_italic: true,
            _linenos: false,
        };
        formatter.setup_styles(&style);
        formatter
    }

    fn setup_styles(&mut self, style: &Style) {
        for (token, attrs) in &style.styles {
            let fg = self.resolve_fg(attrs);
            let bg = self.resolve_bg(attrs);
            let bold = self.use_bold && attrs.bold.unwrap_or(false);
            let underline = self.use_underline && attrs.underline.unwrap_or(false);
            let italic = self.use_italic && attrs.italic.unwrap_or(false);

            let on = ansi_true_color_sequence(fg, bg, bold, underline, italic);
            let off = reset_sequence(fg.is_some(), bg.is_some(), bold || underline || italic);
            self.style_string.insert(*token, (on, off));
        }
    }

    fn resolve_fg(&self, attrs: &StyleAttributes) -> Option<(u8, u8, u8)> {
        if let Some(ref color) = attrs.color {
            return parse_rgb(color);
        }
        None
    }

    fn resolve_bg(&self, attrs: &StyleAttributes) -> Option<(u8, u8, u8)> {
        if let Some(ref bg) = attrs.bg {
            return parse_rgb(bg);
        }
        None
    }

    fn get_style(&self, token: Token) -> Option<&(String, String)> {
        let mut current = token;
        loop {
            if let Some(style) = self.style_string.get(&current) {
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

    fn format_token(&self, ttype: Token, value: &str, outfile: &mut dyn Write) -> std::io::Result<()> {
        let style = self.get_style(ttype);
        let (on, off) = style.cloned().unwrap_or_default();

        let has_fg = !on.is_empty();

        let parts: Vec<&str> = value.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                if has_fg {
                    outfile.write_all(on.as_bytes())?;
                }
                outfile.write_all(part.as_bytes())?;
                if has_fg {
                    outfile.write_all(off.as_bytes())?;
                }
            }
            if i < parts.len() - 1 {
                outfile.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}

impl Formatter for TerminalTrueColorFormatter {
    fn name(&self) -> &str { "TerminalTrueColor" }
    fn extension(&self) -> &str { "" }
    fn mimetype(&self) -> &str { "text/plain" }

    fn format(&self, tokens: &[(Token, String)], outfile: &mut dyn Write) -> std::io::Result<()> {
        for (ttype, value) in tokens {
            if !value.is_empty() {
                self.format_token(*ttype, value, outfile)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xterm_colors() {
        let colors = xterm_colors();
        assert_eq!(colors.len(), 256);
        assert_eq!(colors[0], (0x00, 0x00, 0x00));
        assert_eq!(colors[16], (0x00, 0x00, 0x00));
        assert_eq!(colors[232], (8, 8, 8));
    }

    #[test]
    fn test_closest_color() {
        let palette = xterm_colors();
        assert_eq!(closest_color(0, 0, 0, &palette), 0);
        let idx = closest_color(255, 255, 255, &palette);
        assert!(idx == 15 || idx == 231 || idx == 255);
    }

    #[test]
    fn test_parse_rgb() {
        assert_eq!(parse_rgb("#ff0000"), Some((255, 0, 0)));
        assert_eq!(parse_rgb("#00ff00"), Some((0, 255, 0)));
        assert_eq!(parse_rgb("#0000ff"), Some((0, 0, 255)));
        assert_eq!(parse_rgb("#123456"), Some((0x12, 0x34, 0x56)));
        assert_eq!(parse_rgb("invalid"), None);
    }

    #[test]
    fn test_ansi_256_sequence() {
        let seq = ansi_256_sequence(Some(196), None, false, false, false);
        assert_eq!(seq, "\x1b[38;5;196m");

        let seq = ansi_256_sequence(Some(196), Some(235), true, true, false);
        assert_eq!(seq, "\x1b[38;5;196;48;5;235;01;04m");

        let seq = ansi_256_sequence(None, None, false, false, false);
        assert_eq!(seq, "");
    }

    #[test]
    fn test_terminal256_formatter() {
        let formatter = Terminal256Formatter::new(None);
        let tokens = vec![
            (Token::KEYWORD, "def".to_string()),
            (Token::NAME, " hello".to_string()),
        ];
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("def"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_true_color_formatter() {
        let formatter = TerminalTrueColorFormatter::new(None);
        let tokens = vec![
            (Token::KEYWORD, "def".to_string()),
        ];
        let mut output: Vec<u8> = Vec::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("def"));
    }
}
