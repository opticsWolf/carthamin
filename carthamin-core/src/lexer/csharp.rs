use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// C# lexer for .NET C# source code.
pub struct CSharpLexer {
    inner: RegexLexer,
}

impl CSharpLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("C#");
        inner.aliases = vec!["csharp", "c#", "cs"];
        inner.filenames = vec!["*.cs"];
        inner.mimetypes = vec!["text/x-csharp"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Preprocessor directives
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#\s*(if|endif|else|elif|define|undef|line|error|warning|region|endregion|pragma)", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"@?"(""|[^"])*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'\\.'|'[^\\]'", Token::STRING_CHAR).unwrap(), action: LexerAction::token(Token::STRING_CHAR) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9]+(\.[0-9]*)?([eE][+-][0-9]+)?[flFLdD]?|0[xX][0-9a-fA-F]+[Ll]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators — FIXED: escape + * ? . | that are literal operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r">>>=|>>=|<<=|<=|>=|\+=|-=|\*=|/=|%=|&=|\|=|\^=|\?\?=|=>|\?\?|\.\.\.|\.\.|\.|!=|==|&&|\|\||>>>|>>|<<|\+\+|--|[%&|^~-]|=|[<>]", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\];:,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "abstract", "as", "async", "await", "base", "break", "by", "case", "catch", "checked",
            "const", "continue", "default", "delegate", "do", "else", "enum", "event", "explicit",
            "extern", "false", "finally", "fixed", "for", "foreach", "goto", "if", "implicit", "in",
            "interface", "internal", "is", "let", "lock", "new", "null", "on", "operator", "out",
            "override", "params", "private", "protected", "public", "readonly", "ref", "return",
            "sealed", "sizeof", "stackalloc", "static", "switch", "this", "throw", "true", "try",
            "typeof", "unchecked", "unsafe", "virtual", "void", "while", "get", "set", "partial",
            "yield", "add", "remove", "value", "alias", "ascending", "descending", "from", "group",
            "into", "orderby", "select", "thenby", "where", "join", "equals", "record", "allows",
            "and", "init", "managed", "nameof", "nint", "not", "notnull", "nuint", "or", "scoped",
            "unmanaged", "when", "with", "file", "global",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Type keywords
        let type_keywords = [
            "bool", "byte", "char", "decimal", "double", "dynamic", "float", "int", "long", "object",
            "sbyte", "short", "string", "uint", "ulong", "ushort", "var",
        ];
        let type_pattern = format!(r"\b({})\b\??", type_keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[_a-zA-Z][\w]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        CSharpLexer { inner }
    }
}

impl Lexer for CSharpLexer {
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
    fn test_csharp_keywords() {
        let lexer = CSharpLexer::new();
        let tokens = lexer.get_tokens("class Program { static void Main() { } }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_csharp_comment() {
        let lexer = CSharpLexer::new();
        let tokens = lexer.get_tokens("// this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_csharp_string() {
        let lexer = CSharpLexer::new();
        let tokens = lexer.get_tokens(r#"string s = "hello";"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_csharp_number() {
        let lexer = CSharpLexer::new();
        let tokens = lexer.get_tokens("int x = 42;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }
}
