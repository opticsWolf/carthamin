use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// CSS lexer supporting selectors, properties, values, and at-rules.
pub struct CssLexer {
    inner: RegexLexer,
}

impl CssLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("CSS");
        inner.aliases = vec!["css"];
        inner.filenames = vec!["*.css"];
        inner.mimetypes = vec!["text/css"];

        // Root state
        let mut root_rules = Vec::new();

        // Whitespace
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t\r\n]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // Comment
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // At-rules
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"@(font-face|import|media|page|charset|namespace|keyframes|supports|document|font-document|viewport|-webkit-keyframes|-moz-keyframes|-o-keyframes)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"@[a-zA-Z_-][a-zA-Z0-9_-]*", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // URL function
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"url\([^)]*\)", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Pseudo-classes and pseudo-elements
        root_rules.push(LexerRule { pattern: TokenPattern::new(r":[a-zA-Z_-][a-zA-Z0-9_-]*", Token::NAME_TAG).unwrap(), action: LexerAction::token(Token::NAME_TAG) });

        // Property names
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_-][a-zA-Z0-9_-]*\s*:", Token::NAME_ATTRIBUTE).unwrap(), action: LexerAction::token(Token::NAME_ATTRIBUTE) });

        // Numbers with units
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?[0-9]*\.[0-9]+|[0-9]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}();,]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Identifiers (selectors, values)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_-][a-zA-Z0-9_-]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        CssLexer { inner }
    }
}

impl Lexer for CssLexer {
    fn get_tokens(&self, code: &str) -> Vec<(Token, String)> {
        self.inner.get_tokens(code)
    }

    fn name(&self) -> &str {
        &self.inner.name
    }

    fn aliases(&self) -> &[&str] {
        &self.inner.aliases
    }

    fn filenames(&self) -> &[&str] {
        &self.inner.filenames
    }

    fn mimetypes(&self) -> &[&str] {
        &self.inner.mimetypes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_basic() {
        let lexer = CssLexer::new();
        let tokens = lexer.get_tokens("body { color: red; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_css_property() {
        let lexer = CssLexer::new();
        let tokens = lexer.get_tokens("color: #ff0000;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_ATTRIBUTE));
    }

    #[test]
    fn test_css_comment() {
        let lexer = CssLexer::new();
        let tokens = lexer.get_tokens("/* comment */");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }

    #[test]
    fn test_css_at_rule() {
        let lexer = CssLexer::new();
        let tokens = lexer.get_tokens("@media screen { }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_css_number() {
        let lexer = CssLexer::new();
        let tokens = lexer.get_tokens("width: 100px;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }
}
