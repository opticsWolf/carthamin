use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Julia lexer for Julia source code.
pub struct JuliaLexer {
    inner: RegexLexer,
}

impl JuliaLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Julia");
        inner.aliases = vec!["julia"];
        inner.filenames = vec!["*.jl"];
        inner.mimetypes = vec!["text/x-julia"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Triple-quoted strings — FIXED: use r##"..."## to embed quotes, single backslashes
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##""""[^"]*(?:""|""[^"]|"[^"])*"""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Raw strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##"r"""[^"]*(?:""|""[^"]|"[^"])*"""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"r"([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Number literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[oO][0-7_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d[\d_]*(\.\d[\d_]*([eE][+-]?\d[\d_]*)?)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators — FIXED: use single backslashes in raw string
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|=>|<:|<%|>:|&&|!=|==|<=|>=|<|>|!|~|\||\?|:|=|\+|-|\*|/|%|&|\^", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};:,.\@\#]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "abstract", "as", "begin", "break", "catch", "const", "continue", "do",
            "else", "elseif", "end", "export", "for", "function", "global", "if",
            "in", "local", "macro", "module", "not", "quote", "return", "struct",
            "try", "type", "using", "while", "where", "let", "finally",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Type keywords
        let type_keywords = [
            "Any", "Array", "Bool", "Char", "Dict", "Float32", "Float64", "Int",
            "Int16", "Int32", "Int64", "Int8", "IO", "IOStream", "Integer",
            "Missing", "Nothing", "Number", "Ptr", "Range", "Regex", "String",
            "Symbol", "Tuple", "Type", "UInt", "UInt16", "UInt32", "UInt64",
            "UInt8", "UnitRange", "Vector", "AbstractArray", "AbstractIO",
            "AbstractString", "AbstractType", "AbstractVector",
        ];
        let type_pattern = format!(r"\b({})\b", type_keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        JuliaLexer { inner }
    }
}

impl Lexer for JuliaLexer {
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
    fn test_julia_keywords() {
        let lexer = JuliaLexer::new();
        let tokens = lexer.get_tokens("function hello() println(\"world\") end");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_julia_comment() {
        let lexer = JuliaLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
