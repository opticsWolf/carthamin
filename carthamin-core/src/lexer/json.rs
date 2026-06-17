use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// JSON lexer for JSON data.
pub struct JsonLexer {
    inner: RegexLexer,
}

impl JsonLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("JSON");
        inner.aliases = vec!["json"];
        inner.filenames = vec!["*.json"];
        inner.mimetypes = vec!["application/json", "text/json"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Brackets and punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}\[\],:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Keywords (true, false, null)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(true|false|null)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        inner.states.insert("root".to_string(), root_rules);

        JsonLexer { inner }
    }
}

impl Lexer for JsonLexer {
    fn get_tokens(&self, text: &str) -> Vec<(Token, String)> {
        self.inner.get_tokens(text)
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
    fn test_json_basic() {
        let lexer = JsonLexer::new();
        let tokens = lexer.get_tokens(r#"{"name": "John", "age": 30}"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
        assert!(token_types.contains(&Token::NUMBER));
        assert!(token_types.contains(&Token::PUNCTUATION));
    }

    #[test]
    fn test_json_keywords() {
        let lexer = JsonLexer::new();
        let tokens = lexer.get_tokens(r#"{"active": true, "data": null}"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }
}
