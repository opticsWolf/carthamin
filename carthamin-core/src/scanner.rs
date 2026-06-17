use regex::Regex;
use crate::token::Token;

/// A compiled regex pattern with an associated token type and optional actions.
#[derive(Debug, Clone)]
pub struct TokenPattern {
    pub regex: Regex,
    pub token: Token,
    /// If Some(n), this pattern has n capture groups that should be re-tokenized.
    pub groups: Option<Vec<Token>>,
    /// If true, push a new state when this pattern matches.
    pub push_state: Option<String>,
    /// If true, pop the current state when this pattern matches.
    pub pop_state: bool,
}

impl TokenPattern {
    pub fn new(pattern: &str, token: Token) -> Result<Self, regex::Error> {
        Ok(TokenPattern {
            regex: Regex::new(pattern)?,
            token,
            groups: None,
            push_state: None,
            pop_state: false,
        })
    }

    pub fn with_groups(pattern: &str, groups: Vec<Token>) -> Result<Self, regex::Error> {
        Ok(TokenPattern {
            regex: Regex::new(pattern)?,
            token: Token::TOKEN, // placeholder, groups override
            groups: Some(groups),
            push_state: None,
            pop_state: false,
        })
    }

    pub fn with_push(pattern: &str, token: Token, state: &str) -> Result<Self, regex::Error> {
        Ok(TokenPattern {
            regex: Regex::new(pattern)?,
            token,
            groups: None,
            push_state: Some(state.to_string()),
            pop_state: false,
        })
    }
}

/// A regex scanner that matches patterns against text.
pub struct RegexScanner {
    pub patterns: Vec<TokenPattern>,
}

impl RegexScanner {
    pub fn new() -> Self {
        RegexScanner { patterns: Vec::new() }
    }

    pub fn add(&mut self, pattern: TokenPattern) {
        self.patterns.push(pattern);
    }

    /// Search all patterns against the text at the given position.
    /// Returns the best match: prefer earliest start, then longest, then first defined.
    pub fn search(&self, text: &str, start: usize) -> Option<(usize, usize, &TokenPattern)> {
        let mut best: Option<(usize, usize, &TokenPattern)> = None;

        for pattern in &self.patterns {
            if let Some(m) = pattern.regex.find(&text[start..]) {
                let abs_start = start + m.start();
                let abs_end = start + m.end();

                // Prefer earliest start, then longest match, then first-defined pattern
                match &best {
                    None => best = Some((abs_start, abs_end, pattern)),
                    Some((best_start, best_end, _)) => {
                        if abs_start < *best_start ||
                           (abs_start == *best_start && abs_end > *best_end) {
                            best = Some((abs_start, abs_end, pattern));
                        }
                    }
                }
            }
        }

        best
    }

    /// Get all matching ranges for the text.
    pub fn get_ranges(&self, text: &str) -> Vec<(usize, usize, Token)> {
        let mut ranges = Vec::new();
        let mut pos = 0;

        while pos < text.len() {
            if let Some((start, end, pattern)) = self.search(text, pos) {
                // Skip any text before the match
                if start > pos {
                    ranges.push((pos, start, Token::TEXT));
                }
                ranges.push((start, end, pattern.token));
                pos = if end > pos { end } else { pos + 1 };
            } else {
                // No match — treat rest as text
                ranges.push((pos, text.len(), Token::TEXT));
                break;
            }
        }

        ranges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_basic() {
        let mut scanner = RegexScanner::new();
        scanner.add(TokenPattern::new(r"\d+", Token::NUMBER_INTEGER).unwrap());
        scanner.add(TokenPattern::new(r"[a-zA-Z_]\w*", Token::NAME).unwrap());

        let ranges = scanner.get_ranges("42 hello");
        assert!(!ranges.is_empty());
        // First match should be NUMBER_INTEGER for "42"
        assert_eq!(ranges[0].2, Token::NUMBER_INTEGER);
    }
}
