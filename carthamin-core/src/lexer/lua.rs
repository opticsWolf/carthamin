use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Lua lexer for Lua source code.
pub struct LuaLexer {
    inner: RegexLexer,
}

impl LuaLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Lua");
        inner.aliases = vec!["lua"];
        inner.filenames = vec!["*.lua", "*.wlua"];
        inner.mimetypes = vec!["text/x-lua", "application/x-lua"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Shebang
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#!.*", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Comments — FIXED: Rust regex doesn't support backreferences (\1).
        // Use patterns that match common cases without backreference validation.
        // Multiline comment: --[[...]] or --[=[...]=] etc.
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"--\[=*\[.*?\]=*\]", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"--[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });
        // Long strings — without backreference, match common cases
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\[=*\[.*?\]=*\]", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][\da-fA-F]*(\.[\da-fA-F]*)?(p[+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(\d*\.\d+|\d+\.\d*)(e[+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d+e[+-]?\d+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(and|or|not)\b", Token::OPERATOR_WORD).unwrap(), action: LexerAction::token(Token::OPERATOR_WORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[=<>|~&+\-*/%#^]+|\.\.", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[\[\]{}():;,.]+", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "break", "do", "else", "elseif", "end", "for", "if", "in",
            "repeat", "return", "then", "until", "while", "function", "local",
            "goto", "as",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtins
        let builtins = [
            "_G", "_VERSION", "assert", "collectgarbage", "dofile", "error",
            "getmetatable", "input", "next", "pairs", "pcall", "print",
            "rawget", "rawset", "require", "select", "setmetatable", "tonumber",
            "tostring", "type", "unpack", "warn", "byte", "char", "concat",
            "copy", "find", "format", "gsub", "len", "lower", "match", "rep",
            "reverse", "sub", "upper", "close", "flush", "lines", "open",
            "read", "seek", "setvbuf", "write", "clock", "date", "difftime",
            "time", "exit", "getenv", "setenv", "remove", "rename", "tmpfile",
            "io", "math", "os", "string", "table", "bit32", "coroutine",
        ];
        let builtin_pattern = format!(r"\b({})\b", builtins.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&builtin_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        LuaLexer { inner }
    }
}

impl Lexer for LuaLexer {
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
    fn test_lua_keywords() {
        let lexer = LuaLexer::new();
        let tokens = lexer.get_tokens("if x == 1 then print(x) end");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_lua_comment() {
        let lexer = LuaLexer::new();
        let tokens = lexer.get_tokens("-- comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_lua_string() {
        let lexer = LuaLexer::new();
        let tokens = lexer.get_tokens(r#"local s = "hello""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }
}
