use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Protocol Buffer lexer for .proto files.
pub struct ProtoBufLexer {
    inner: RegexLexer,
}

impl ProtoBufLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Protocol Buffer");
        inner.aliases = vec!["protobuf", "proto"];
        inner.filenames = vec!["*.proto"];
        inner.mimetypes = vec!["text/x-protobuf"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Keywords
        let keywords = [
            "syntax", "package", "import", "option", "message", "enum", "service",
            "rpc", "returns", "oneof", "map", "reserved", "extensions", "extend",
            "packed", "optional", "required", "repeated", "group",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Preprocessor directives - use hex escapes for quotes
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^\s*syntax\s*=\s*[\x27\x22]proto3[\x27\x22]", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}();=\[\]]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        inner.states.insert("root".to_string(), root_rules);

        ProtoBufLexer { inner }
    }
}

impl Lexer for ProtoBufLexer {
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
    fn test_protobuf_basic() {
        let lexer = ProtoBufLexer::new();
        let tokens = lexer.get_tokens("syntax = \"proto3\";\nmessage User { string name = 1; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_protobuf_comment() {
        let lexer = ProtoBufLexer::new();
        let tokens = lexer.get_tokens("// comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
