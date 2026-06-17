use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Triple double-quote pattern for Python strings.
const TRIPLE_DQ: &str = r#""""#;

/// Python lexer supporting keywords, strings, comments, numbers, operators, and names.
/// Token granularity matches Pygments' PythonLexer output.
pub struct PythonLexer {
    inner: RegexLexer,
}

impl PythonLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Python");
        inner.aliases = vec!["python", "py", "python3", "py3"];
        inner.filenames = vec!["*.py", "*.pyw", "*.pyi"];
        inner.mimetypes = vec!["text/x-python", "application/x-python"];

        // Root state
        let mut root_rules = Vec::new();

        // Whitespace
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Line continuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\\\n", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // Triple-quoted strings (docstrings) - triple double-quoted
        root_rules.push(LexerRule { pattern: TokenPattern::new(TRIPLE_DQ, Token::STRING_DOC).unwrap(), action: LexerAction::push("tdqs") });
        // Triple-quoted strings (docstrings) - triple single-quoted
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'''", Token::STRING_DOC).unwrap(), action: LexerAction::push("tsqs") });

        // String prefixes with triple double-quotes (prefix + opening quotes consumed together)
        root_rules.push(LexerRule { pattern: TokenPattern::new(&format!("[rRbB]{{0,2}}{}", TRIPLE_DQ), Token::STRING_DOC).unwrap(), action: LexerAction::push("tdqs") });
        // String prefixes with triple single-quotes
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[rRbB]{0,2}'''", Token::STRING_DOC).unwrap(), action: LexerAction::push("tsqs") });

        // f-strings triple double-quoted
        root_rules.push(LexerRule { pattern: TokenPattern::new(&format!("(?i)[fF]{}", TRIPLE_DQ), Token::STRING_DOC).unwrap(), action: LexerAction::push("tdqf") });
        // f-strings triple single-quoted
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(?i)[fF]'''", Token::STRING_DOC).unwrap(), action: LexerAction::push("tsqf") });

        // f-strings single/double quoted (prefix + opening quote consumed together)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##"(?i)[fF]""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("dqf") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(?i)[fF]'", Token::STRING_SINGLE).unwrap(), action: LexerAction::push("sqf") });

        // String prefixes with single/double quotes (prefix + opening quote consumed together)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##"[rRbB]{0,2}""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("dqs") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[rRbB]{0,2}'", Token::STRING_SINGLE).unwrap(), action: LexerAction::push("sqs") });

        // Keywords (not def/class/from/import which have special handling below)
        let keywords = [
            "False", "None", "True", "as", "assert", "async", "await",
            "break", "continue", "del", "elif", "else",
            "except", "finally", "for", "global", "if",
            "lambda", "nonlocal", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ];
        let kw_pattern = format!(r"({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&kw_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // def and class (push to name states for granular token types)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(def)\b", Token::KEYWORD).unwrap(), action: LexerAction::push("funcname") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(class)\b", Token::KEYWORD).unwrap(), action: LexerAction::push("classname") });

        // from ... import / import
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(from)\b", Token::KEYWORD_NAMESPACE).unwrap(), action: LexerAction::push("fromimport") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(import)\b", Token::KEYWORD_NAMESPACE).unwrap(), action: LexerAction::push("import") });

        // Numbers - must be before operators (because of 0x, 0b prefixes)
        // Float: digits.digits with optional exponent
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d+\.\d+([eE][+-]?\d+)?", Token::NUMBER_FLOAT).unwrap(), action: LexerAction::token(Token::NUMBER_FLOAT) });
        // Hex: 0x...
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+", Token::NUMBER_HEX).unwrap(), action: LexerAction::token(Token::NUMBER_HEX) });
        // Binary: 0b...
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+", Token::NUMBER_BIN).unwrap(), action: LexerAction::token(Token::NUMBER_BIN) });
        // Integer (plain digits, no dot)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d+", Token::NUMBER_INTEGER).unwrap(), action: LexerAction::token(Token::NUMBER_INTEGER) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"!=|==|<<|>>|:=|[-~+/*%<>&^|.]", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Operator words (in, is, and, or, not) - must be after keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(in|is|and|or|not)\b", Token::OPERATOR_WORD).unwrap(), action: LexerAction::token(Token::OPERATOR_WORD) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[]{}:(),;]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Builtin functions
        let builtins = [
            "abs", "all", "any", "bin", "bool", "breakpoint",
            "bytearray", "bytes", "callable", "chr", "classmethod", "compile", "complex",
            "delattr", "dict", "dir", "divmod", "enumerate", "eval", "exec", "filter",
            "float", "format", "frozenset", "getattr", "globals", "hasattr", "hash",
            "help", "hex", "id", "input", "int", "isinstance", "issubclass", "iter",
            "len", "list", "locals", "map", "max", "memoryview", "min", "next", "object",
            "oct", "open", "ord", "pow", "print", "property", "range", "repr", "reversed",
            "round", "set", "setattr", "slice", "sorted", "staticmethod", "str", "sum",
            "super", "tuple", "type", "vars", "zip",
        ];
        let builtin_pattern = format!(r"({})\b", builtins.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&builtin_pattern, Token::NAME_BUILTIN).unwrap(), action: LexerAction::token(Token::NAME_BUILTIN) });

        // Decorator: @name
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"@[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME_DECORATOR).unwrap(), action: LexerAction::token(Token::NAME_DECORATOR) });

        // Names (identifiers)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        // Function name state (after 'def')
        inner.states.insert("funcname".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME_FUNCTION).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Class name state (after 'class')
        inner.states.insert("classname".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME_CLASS).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Import state
        inner.states.insert("import".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_.]*", Token::NAME_NAMESPACE).unwrap(), action: LexerAction::token(Token::NAME_NAMESPACE) },
            LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // From import state
        inner.states.insert("fromimport".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_.]*", Token::NAME_NAMESPACE).unwrap(), action: LexerAction::token(Token::NAME_NAMESPACE) },
            LexerRule { pattern: TokenPattern::new(r"(import)\b", Token::KEYWORD_NAMESPACE).unwrap(), action: LexerAction::push("import") },
            LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // === String content states ===
        // Opening quote already consumed by root rules, these states handle content + closing quote

        // Triple double-quoted string content
        inner.states.insert("tdqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(TRIPLE_DQ, Token::STRING_DOC).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\]+"##, Token::STRING_DOC).unwrap(), action: LexerAction::token(Token::STRING_DOC) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
        ]);

        // Triple single-quoted string content
        inner.states.insert("tsqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"'''", Token::STRING_DOC).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[^'\\]+", Token::STRING_DOC).unwrap(), action: LexerAction::token(Token::STRING_DOC) },
            LexerRule { pattern: TokenPattern::new(r"\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
        ]);

        // Double-quoted string content (after prefix + opening quote consumed)
        inner.states.insert("dqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r##""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\]+"##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
        ]);

        // Single-quoted string content (after prefix + opening quote consumed)
        inner.states.insert("sqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"'", Token::STRING_SINGLE).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[^'\\]+", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
        ]);

        // f-string triple double-quoted content
        inner.states.insert("tdqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_INTERPOL).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\{]+"##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(TRIPLE_DQ, Token::STRING_DOC).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string triple single-quoted content
        inner.states.insert("tsqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_INTERPOL).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r"[^'\\{]+", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r"'''", Token::STRING_DOC).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string double-quoted content (after prefix + opening quote consumed)
        inner.states.insert("dqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_INTERPOL).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\{]+"##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string single-quoted content (after prefix + opening quote consumed)
        inner.states.insert("sqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_INTERPOL).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r"[^'\\{]+", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r"'", Token::STRING_SINGLE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string expression (simplified)
        inner.states.insert("fstring-expr".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\}", Token::STRING_INTERPOL).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r":[^}]*", Token::STRING_INTERPOL).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[^}]+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
        ]);

        PythonLexer { inner }
    }
}

impl Lexer for PythonLexer {
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
    fn test_python_keywords() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("if x == 1: pass");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_python_string() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens(r#"hello = "world""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_python_comment() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("# this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_python_function() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("def foo(): pass");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME_FUNCTION));
    }

    #[test]
    fn test_python_class() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("class Foo: pass");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME_CLASS));
    }

    #[test]
    fn test_python_fstring() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens(r#"f"hello {name}""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
        assert!(token_types.contains(&Token::STRING_INTERPOL));
    }

    #[test]
    fn test_python_number_integer() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("x = 42");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER_INTEGER));
    }

    #[test]
    fn test_python_number_float() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("x = 3.14");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER_FLOAT));
    }

    #[test]
    fn test_python_number_hex() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("x = 0xFF");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER_HEX));
    }

    #[test]
    fn test_python_number_bin() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("x = 0b1010");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER_BIN));
    }

    #[test]
    fn test_python_triple_quoted() {
        let lexer = PythonLexer::new();
        let code = "\"\"\"docstring\"\"\"";
        let tokens = lexer.get_tokens(code);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOC));
    }

    #[test]
    fn test_python_builtin() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("print('hello')");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_BUILTIN));
    }

    #[test]
    fn test_python_decorator() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("@decorator\ndef foo(): pass");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_DECORATOR));
    }

    #[test]
    fn test_python_operator_words() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("x and y");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR_WORD));
    }
}
