use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Go lexer supporting Go 1.20+ features.
pub struct GoLexer {
    inner: RegexLexer,
}

impl GoLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Go");
        inner.aliases = vec!["go", "golang"];
        inner.filenames = vec!["*.go"];
        inner.mimetypes = vec!["text/x-gosrc"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Numeric literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[oO][0-7_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*\.[0-9_]+([eE][+-]?[0-9_]+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9_]+\.[0-9_]*([eE][+-]?[0-9_]+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Imaginary numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9]+[0-9_]*i", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|\.\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]=?|[<>]=?|<<=?|>>=?|&=|&&|\|=|\|\||\^=|\!=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"==|!=|<|>|<=|>=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(break|case|chan|const|continue|default|defer|else|fallthrough|for|func|go|goto|if|import|interface|map|package|range|return|select|struct|switch|type|var)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtins
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(bool|byte|complex64|complex128|error|float32|float64|int|int8|int16|int32|int64|rune|string|uint|uint8|uint16|uint32|uint64|uintptr|any|comparable)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(make|len|cap|append|close|delete|copy|new|panic|recover|print|println|real|imag|complex|iota|nil|true|false)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Raw strings (backtick)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"`[^`]*`", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Characters
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        GoLexer { inner }
    }
}

impl Lexer for GoLexer {
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
    fn test_go_basic() {
        let lexer = GoLexer::new();
        let tokens = lexer.get_tokens("package main\nfunc main() { fmt.Println(\"hello\") }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_go_comment() {
        let lexer = GoLexer::new();
        let tokens = lexer.get_tokens("// this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_go_raw_string() {
        let lexer = GoLexer::new();
        let tokens = lexer.get_tokens("s := `raw string`");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_go_number() {
        let lexer = GoLexer::new();
        let tokens = lexer.get_tokens("x := 42");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }
}
