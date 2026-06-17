use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// YAML lexer for YAML data.
pub struct YamlLexer {
    inner: RegexLexer,
}

impl YamlLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("YAML");
        inner.aliases = vec!["yaml"];
        inner.filenames = vec!["*.yaml", "*.yml"];
        inner.mimetypes = vec!["text/x-yaml", "application/x-yaml"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and indentation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Document markers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^---[ \t]*$", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^\.\.\.[ \t]*$", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Strings (single and double quoted)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Plain scalar — any characters except special YAML chars
        // FIXED: removed invalid \| and \- escapes; put - at end of class; use \[\] for brackets
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[^\n#%&*>|<,\[\]{}-]+\n", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[:\[\]{}]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(true|false|null)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        inner.states.insert("root".to_string(), root_rules);

        YamlLexer { inner }
    }
}

impl Lexer for YamlLexer {
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
    fn test_yaml_basic() {
        let lexer = YamlLexer::new();
        let tokens = lexer.get_tokens("name: John\nage: 30");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
        assert!(token_types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_yaml_comment() {
        let lexer = YamlLexer::new();
        let tokens = lexer.get_tokens("# this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
