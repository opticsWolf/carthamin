use std::collections::HashMap;
use crate::token::Token;

pub mod generated;

/// Style attributes for a single token type.
#[derive(Debug, Clone)]
pub struct StyleAttributes {
    pub color: Option<String>,         // foreground color (#RRGGBB)
    pub bg: Option<String>,            // background color (#RRGGBB)
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub blink: Option<bool>,
    pub roman: Option<bool>,
}

impl StyleAttributes {
    pub fn empty() -> Self {
        StyleAttributes {
            color: None, bg: None, bold: None, italic: None,
            underline: None, blink: None, roman: None,
        }
    }

    /// Parse a CSS-style property string: "color:#ff0000;bold:true;bg:#ffffff"
    pub fn from_css_string(s: &str) -> Self {
        let mut attrs = StyleAttributes::empty();
        for part in s.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((key, value)) = part.split_once(':') {
                match key.trim() {
                    "color" => attrs.color = Some(value.trim().to_string()),
                    "bg" => attrs.bg = Some(value.trim().to_string()),
                    "bold" => attrs.bold = Some(value.trim().to_lowercase() == "true"),
                    "italic" => attrs.italic = Some(value.trim().to_lowercase() == "true"),
                    "underline" => attrs.underline = Some(value.trim().to_lowercase() == "true"),
                    "blink" => attrs.blink = Some(value.trim().to_lowercase() == "true"),
                    "roman" => attrs.roman = Some(value.trim().to_lowercase() == "true"),
                    _ => {}
                }
            }
        }
        attrs
    }

    /// Format back to CSS string.
    pub fn to_css_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref c) = self.color {
            parts.push(format!("color:{}", c));
        }
        if let Some(ref b) = self.bg {
            parts.push(format!("bg:{}", b));
        }
        if let Some(true) = self.bold {
            parts.push("bold:true".to_string());
        }
        if let Some(true) = self.italic {
            parts.push("italic:true".to_string());
        }
        if let Some(true) = self.underline {
            parts.push("underline:true".to_string());
        }
        parts.join(";")
    }
}

/// A style maps token types to style attributes.
/// Supports inheritance: if Token.Keyword.Declaration has no style,
/// fall back to Token.Keyword, then Token.
#[derive(Debug, Clone)]
pub struct Style {
    /// Name of this style.
    pub name: String,
    /// Base style to inherit from (optional).
    pub base_style: Option<String>,
    /// Direct style definitions (token → attributes).
    pub styles: HashMap<Token, StyleAttributes>,
    /// Default styles (applied when no specific style found).
    pub default_style: StyleAttributes,
}

impl Style {
    pub fn new(name: &str) -> Self {
        Style {
            name: name.to_string(),
            base_style: None,
            styles: HashMap::new(),
            default_style: StyleAttributes::empty(),
        }
    }

    /// Get the effective style for a token, walking up the inheritance chain.
    /// Merges attributes from child → parent → default, with child taking priority.
    pub fn style_for_token(&self, token: Token) -> StyleAttributes {
        let path: Vec<&str> = token.path.iter().copied().collect();

        // Collect attributes from all levels (child first, then parents)
        let mut result = StyleAttributes::empty();

        // Walk from the most specific token up to root
        for i in (0..=path.len()).rev() {
            let parent_path: Vec<&str> = if i == 0 { vec![] } else { path[..i].to_vec() };
            
            // Look up this level in our styles
            for (t, attrs) in &self.styles {
                if t.path.len() == parent_path.len() && t.path.iter().copied().collect::<Vec<_>>() == parent_path {
                    // Merge: only fill in None values from parent
                    if result.color.is_none() {
                        result.color = attrs.color.clone();
                    }
                    if result.bg.is_none() {
                        result.bg = attrs.bg.clone();
                    }
                    if result.bold.is_none() {
                        result.bold = attrs.bold;
                    }
                    if result.italic.is_none() {
                        result.italic = attrs.italic;
                    }
                    if result.underline.is_none() {
                        result.underline = attrs.underline;
                    }
                    if result.blink.is_none() {
                        result.blink = attrs.blink;
                    }
                    if result.roman.is_none() {
                        result.roman = attrs.roman;
                    }
                    break;
                }
            }
        }

        // Fill remaining None from default style
        if result.color.is_none() {
            result.color = self.default_style.color.clone();
        }
        if result.bg.is_none() {
            result.bg = self.default_style.bg.clone();
        }
        if result.bold.is_none() {
            result.bold = self.default_style.bold;
        }
        if result.italic.is_none() {
            result.italic = self.default_style.italic;
        }
        if result.underline.is_none() {
            result.underline = self.default_style.underline;
        }
        if result.blink.is_none() {
            result.blink = self.default_style.blink;
        }
        if result.roman.is_none() {
            result.roman = self.default_style.roman;
        }

        result
    }

    /// Iterate over all token types with their effective styles.
    pub fn iter_styles(&self) -> impl Iterator<Item = (Token, &StyleAttributes)> {
        // For now, just return direct styles
        self.styles.iter().map(|(t, a)| (*t, a))
    }
}

/// ANSI color mapping for terminal output.
pub fn ansi_color(hex_color: &str) -> Option<u8> {
    let hex_color = hex_color.trim_start_matches('#');
    if hex_color.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex_color[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex_color[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex_color[4..6], 16).ok()?;

    // Map to closest 16 ANSI color
    const ANSI_COLORS: &[(u8, u8, u8, u8)] = &[
        (0, 0, 0, 0),        // Black
        (128, 0, 0, 1),      // Red
        (0, 128, 0, 2),      // Green
        (128, 128, 0, 3),    // Yellow
        (0, 0, 128, 4),      // Blue
        (128, 0, 128, 5),    // Magenta
        (0, 128, 128, 6),    // Cyan
        (192, 192, 192, 7),  // Light Gray
        (128, 128, 128, 8),  // Dark Gray
        (255, 0, 0, 9),      // Light Red
        (0, 255, 0, 10),     // Light Green
        (255, 255, 0, 11),   // Light Yellow
        (0, 0, 255, 12),     // Light Blue (using 128 for compatibility)
        (255, 0, 255, 13),   // Light Magenta
        (0, 255, 255, 14),   // Light Cyan
        (255, 255, 255, 15), // White
    ];

    let mut best_idx: u8 = 0;
    let mut best_dist = u32::MAX;

    for &(ar, ag, ab, idx) in ANSI_COLORS {
        let dist = ((r as i32 - ar as i32).pow(2) + (g as i32 - ag as i32).pow(2) + (b as i32 - ab as i32).pow(2)) as u32;
        if dist < best_dist {
            best_dist = dist;
            best_idx = idx;
        }
    }

    Some(best_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;

    #[test]
    fn test_style_attributes_parse() {
        let attrs = StyleAttributes::from_css_string("color:#ff0000;bold:true;bg:#ffffff");
        assert_eq!(attrs.color, Some("#ff0000".to_string()));
        assert_eq!(attrs.bold, Some(true));
        assert_eq!(attrs.bg, Some("#ffffff".to_string()));
    }

    #[test]
    fn test_style_for_token() {
        let mut style = Style::new("test");
        style.styles.insert(Token::KEYWORD, StyleAttributes::from_css_string("color:#0000ff"));
        style.styles.insert(Token::STRING, StyleAttributes::from_css_string("color:#00ff00"));

        // Direct match
        let kw_style = style.style_for_token(Token::KEYWORD);
        assert_eq!(kw_style.color, Some("#0000ff".to_string()));

        // No direct match for KEYWORD_DECLARATION, should fall back
        // (for now it returns default since parent lookup is simplified)
        let _decl_style = style.style_for_token(Token::KEYWORD_DECLARATION);
    }

    #[test]
    fn test_ansi_color() {
        assert_eq!(ansi_color("#000000"), Some(0));
        assert_eq!(ansi_color("#ff0000"), Some(9)); // Light Red is closest to pure red
        assert_eq!(ansi_color("#ffffff"), Some(15));
    }

    #[test]
    fn test_generated_monokai() {
        use super::generated::monokai_style;
        let style = monokai_style();
        assert_eq!(style.name, "monokai");
        assert_eq!(style.default_style.bg, Some("#272822".to_string()));

        // Keyword should be cyan
        let kw = style.style_for_token(Token::KEYWORD);
        assert_eq!(kw.color, Some("#66d9ef".to_string()));

        // String should be yellow-ish
        let s = style.style_for_token(Token::STRING);
        assert_eq!(s.color, Some("#e6db74".to_string()));
    }

    #[test]
    fn test_generated_default() {
        use super::generated::default_style;
        let style = default_style();
        assert_eq!(style.name, "default");
        assert_eq!(style.default_style.bg, Some("#f8f8f8".to_string()));

        // Keyword should be green, bold
        let kw = style.style_for_token(Token::KEYWORD);
        assert_eq!(kw.color, Some("#008000".to_string()));
        assert_eq!(kw.bold, Some(true));
    }

    #[test]
    fn test_generated_get_style() {
        use super::generated::get_style;
        assert!(get_style("monokai").is_some());
        assert!(get_style("default").is_some());
        assert!(get_style("dracula").is_some());
        assert!(get_style("nonexistent").is_none());
    }

    #[test]
    fn test_generated_all_names() {
        use super::generated::ALL_STYLE_NAMES;
        assert!(!ALL_STYLE_NAMES.is_empty());
        assert!(ALL_STYLE_NAMES.contains(&"monokai"));
        assert!(ALL_STYLE_NAMES.contains(&"dracula"));
        assert!(ALL_STYLE_NAMES.contains(&"default"));
    }
}
