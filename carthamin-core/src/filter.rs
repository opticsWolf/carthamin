use crate::token::Token;

/// Base trait for token stream filters.
pub trait Filter {
    /// Name of this filter.
    fn name(&self) -> &str;

    /// Apply the filter to a token stream.
    fn apply(&self, tokens: &[(Token, String)]) -> Vec<(Token, String)>;
}

/// Collapse consecutive whitespace into single spaces.
pub struct CollapseWhitespaceFilter {
    pub ignore: bool,
}

impl CollapseWhitespaceFilter {
    pub fn new(ignore: bool) -> Self {
        CollapseWhitespaceFilter { ignore }
    }
}

impl Filter for CollapseWhitespaceFilter {
    fn name(&self) -> &str {
        "collapse"
    }

    fn apply(&self, tokens: &[(Token, String)]) -> Vec<(Token, String)> {
        let mut result = Vec::new();
        let mut in_whitespace = false;

        for (token, text) in tokens {
            if *token == Token::WHITESPACE || *token == Token::TEXT {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    if !self.ignore && !in_whitespace {
                        result.push((*token, " ".to_string()));
                        in_whitespace = true;
                    }
                } else {
                    in_whitespace = false;
                    result.push((*token, text.clone()));
                }
            } else {
                in_whitespace = false;
                result.push((*token, text.clone()));
            }
        }

        result
    }
}

/// Convert token text to uppercase or lowercase.
pub struct KeywordCaseFilter {
    pub case: String, // "upper" or "lower"
}

impl KeywordCaseFilter {
    pub fn new(case: &str) -> Self {
        KeywordCaseFilter {
            case: case.to_string(),
        }
    }
}

impl Filter for KeywordCaseFilter {
    fn name(&self) -> &str {
        "keywordcase"
    }

    fn apply(&self, tokens: &[(Token, String)]) -> Vec<(Token, String)> {
        tokens.iter().map(|(token, text)| {
            if *token == Token::KEYWORD || token.is_subtype_of(&Token::KEYWORD) {
                let transformed = if self.case == "upper" {
                    text.to_uppercase()
                } else {
                    text.to_lowercase()
                };
                (*token, transformed)
            } else {
                (*token, text.clone())
            }
        }).collect()
    }
}

/// Make whitespace visible by replacing with special characters.
pub struct VisibleWhitespaceFilter {
    pub spaces: String,
    pub tabs: String,
    pub tabsize: usize,
}

impl VisibleWhitespaceFilter {
    pub fn new() -> Self {
        VisibleWhitespaceFilter {
            spaces: "·".to_string(),
            tabs: "→".to_string(),
            tabsize: 8,
        }
    }
}

impl Filter for VisibleWhitespaceFilter {
    fn name(&self) -> &str {
        "whitespace"
    }

    fn apply(&self, tokens: &[(Token, String)]) -> Vec<(Token, String)> {
        tokens.iter().map(|(token, text)| {
            let replaced = text
                .replace(' ', &self.spaces)
                .replace('\t', &self.tabs.repeat(self.tabsize));
            (*token, replaced)
        }).collect()
    }
}

/// Strip comments from the token stream.
pub struct StripCommentsFilter;

impl Filter for StripCommentsFilter {
    fn name(&self) -> &str {
        "stripcomments"
    }

    fn apply(&self, tokens: &[(Token, String)]) -> Vec<(Token, String)> {
        tokens.iter()
            .filter(|(token, _)| {
                !(*token == Token::COMMENT || token.is_subtype_of(&Token::COMMENT))
            })
            .cloned()
            .collect()
    }
}

/// Strip strings from the token stream.
pub struct StripStringsFilter;

impl Filter for StripStringsFilter {
    fn name(&self) -> &str {
        "stripstrings"
    }

    fn apply(&self, tokens: &[(Token, String)]) -> Vec<(Token, String)> {
        tokens.iter()
            .filter(|(token, _)| {
                !(*token == Token::STRING || token.is_subtype_of(&Token::STRING))
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_whitespace() {
        let filter = CollapseWhitespaceFilter::new(false);
        let tokens = vec![
            (Token::KEYWORD, "if".to_string()),
            (Token::WHITESPACE, "   ".to_string()),
            (Token::NAME, "x".to_string()),
        ];
        let result = filter.apply(&tokens);
        assert_eq!(result[1].1, " ");
    }

    #[test]
    fn test_keyword_case() {
        let filter = KeywordCaseFilter::new("upper");
        let tokens = vec![
            (Token::KEYWORD, "if".to_string()),
            (Token::NAME, "x".to_string()),
        ];
        let result = filter.apply(&tokens);
        assert_eq!(result[0].1, "IF");
        assert_eq!(result[1].1, "x");
    }

    #[test]
    fn test_strip_comments() {
        let filter = StripCommentsFilter;
        let tokens = vec![
            (Token::KEYWORD, "if".to_string()),
            (Token::COMMENT_SINGLE, "# comment".to_string()),
            (Token::NAME, "x".to_string()),
        ];
        let result = filter.apply(&tokens);
        assert_eq!(result.len(), 2);
        assert!(!result.iter().any(|(t, _)| *t == Token::COMMENT_SINGLE));
    }
}
