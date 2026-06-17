use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Python lexer supporting keywords, strings, comments, numbers, operators, and names.
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

        // Triple-quoted strings (docstrings)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"[rRbB]{0,2}""""#, Token::STRING).unwrap(), action: LexerAction::push("tdqs") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[rRbB]{0,2}'''", Token::STRING).unwrap(), action: LexerAction::push("tsqs") });

        // Single/double quoted strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##"[rRbB]{0,2}""##, Token::STRING).unwrap(), action: LexerAction::push("dqs") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[rRbB]{0,2}'", Token::STRING).unwrap(), action: LexerAction::push("sqs") });

        // f-strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"(?i)[fF]""""#, Token::STRING).unwrap(), action: LexerAction::push("tdqf") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(?i)[fF]'''", Token::STRING).unwrap(), action: LexerAction::push("tsqf") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##"(?i)[fF]""##, Token::STRING).unwrap(), action: LexerAction::push("dqf") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(?i)[fF]'", Token::STRING).unwrap(), action: LexerAction::push("sqf") });

        // Keywords
        let keywords = [
            "False", "None", "True", "and", "as", "assert", "async", "await",
            "break", "continue", "del", "elif", "else",
            "except", "finally", "for", "global", "if",
            "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ];
        let kw_pattern = format!(r"({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&kw_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // def and class
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(def)\b", Token::KEYWORD).unwrap(), action: LexerAction::push("funcname") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(class)\b", Token::KEYWORD).unwrap(), action: LexerAction::push("classname") });

        // from ... import / import
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(from)\b", Token::KEYWORD_NAMESPACE).unwrap(), action: LexerAction::push("fromimport") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(import)\b", Token::KEYWORD_NAMESPACE).unwrap(), action: LexerAction::push("import") });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d+\.\d+([eE][+-]?\d+)?j?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\d+([eE][+-]?\d+)?j?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"!=|==|<<|>>|:=|[-~+/*%<>&^|.]", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Operator words
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
        root_rules.push(LexerRule { pattern: TokenPattern::new(&builtin_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Decorator
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"@\s*", Token::NAME).unwrap(), action: LexerAction::push("decorator") });

        // Names (identifiers)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        // Function name state
        inner.states.insert("funcname".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Class name state
        inner.states.insert("classname".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::pop(1) },
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

        // Triple double-quoted string
        inner.states.insert("tdqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r#"""""#, Token::STRING).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\]+"##, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
        ]);
        // Triple single-quoted string
        inner.states.insert("tsqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[^'\\\\]+", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r"\\\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r"'''", Token::STRING).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Double-quoted string
        inner.states.insert("dqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r##"[^"\\]+"##, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Single-quoted string
        inner.states.insert("sqs".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[^'\\\\]+", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r"\\\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string triple double-quoted
        inner.states.insert("tdqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\{]+"##, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r#""""#, Token::STRING).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string triple single-quoted
        inner.states.insert("tsqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r"[^'\\\\{]+", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r"\\\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r"'''", Token::STRING).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string double-quoted
        inner.states.insert("dqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r##"[^"\\{]+"##, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r##"\\.|""##, Token::STRING_ESCAPE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string single-quoted
        inner.states.insert("sqf".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\{", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("fstring-expr") },
            LexerRule { pattern: TokenPattern::new(r"[^'\\\\{]+", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
            LexerRule { pattern: TokenPattern::new(r"\\\\.|'", Token::STRING_ESCAPE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // f-string expression (simplified)
        inner.states.insert("fstring-expr".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\}", Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r":[^}]*", Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[^}]+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
        ]);

        // Decorator state
        inner.states.insert("decorator".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::pop(1) },
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
        assert!(token_types.contains(&Token::STRING));
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
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_python_fstring() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens(r#"f"hello {name}""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_python_number() {
        let lexer = PythonLexer::new();
        let tokens = lexer.get_tokens("x = 42");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_python_triple_quoted() {
        let lexer = PythonLexer::new();
        let code = "\"\"\"docstring\"\"\"";
        let tokens = lexer.get_tokens(code);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        for (t, text) in &tokens {
            println!("Token: {:?}, Text: {:?}", t, text);
        }
        assert!(token_types.contains(&Token::STRING), "Expected STRING token, got: {:?}", token_types);
    }
}
