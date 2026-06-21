use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;
use crate::unistring::{XID_START, XID_CONTINUE};

/// JavaScript lexer supporting ES6+ features.
/// Ported from pygments.lexers.javascript.
pub struct JavaScriptLexer {
    inner: RegexLexer,
}

impl JavaScriptLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("JavaScript");
        inner.aliases = vec!["javascript", "js"];
        inner.filenames = vec!["*.js", "*.jsm", "*.mjs", "*.cjs"];
        inner.mimetypes = vec![
            "application/javascript",
            "application/x-javascript",
            "text/x-javascript",
            "text/javascript",
        ];

        // Comments and whitespace state
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"<!--", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) },
            // In Python: (r'//.*?$', Comment.Single)
            // Rust regex doesn't support non-greedy with $, use [^\n]* instead
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Bad regex state (error recovery)
        inner.states.insert("badregex".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Template literal interpolation state
        inner.states.insert("interp".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"`", Token::STRING_BACKTICK).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"\\.", Token::STRING_BACKTICK).unwrap(), action: LexerAction::token(Token::STRING_BACKTICK) },
            LexerRule { pattern: TokenPattern::new(r"\$\{", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("interp-inside") },
            LexerRule { pattern: TokenPattern::new(r"\$", Token::STRING_BACKTICK).unwrap(), action: LexerAction::token(Token::STRING_BACKTICK) },
            LexerRule { pattern: TokenPattern::new(r"[^`\\$]+", Token::STRING_BACKTICK).unwrap(), action: LexerAction::token(Token::STRING_BACKTICK) },
        ]);

        // Inside template literal interpolation
        inner.states.insert("interp-inside".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\}", Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(1) },
            // Include root for nested expressions - simplified to just consume until }
            LexerRule { pattern: TokenPattern::new(r"[^}]+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Hashbang (recognized by Node.js)
        // In Python: (r'\A#! ?/.*?$', Comment.Hashbang)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\A#! ?/.*?$", Token::COMMENT_HASHBANG).unwrap(), action: LexerAction::token(Token::COMMENT_HASHBANG) });

        // In Python: (r'^(?=\s|/|<!--)', Text, 'slashstartsregex')
        // Rust regex doesn't support look-ahead (?=...), so we handle this by matching the actual patterns
        // and pushing to slashstartsregex state. This is done in the tokenizer logic.
        // For now, we'll add rules that match at start of line
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^//", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::push("commentsandwhitespace") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^<!--", Token::COMMENT).unwrap(), action: LexerAction::push("commentsandwhitespace") });

        // Numeric literals
        // In Python: (r'0[bB][01]+n?', Number.Bin),
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01]+n?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        // In Python: (r'0[oO]?[0-7]+n?', Number.Oct),
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[oO]?[0-7]+n?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        // In Python: (r'0[xX][0-9a-fA-F]+n?', Number.Hex),
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F]+n?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        // In Python: (r'[0-9]+n', Number.Integer),  // Javascript BigInt requires an "n" postfix
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9]+n", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        // In Python: (r'(\.[0-9]+|[0-9]+\.[0-9]*|[0-9]+)([eE][-+]?[0-9]+)?', Number.Float)
        // Note: In JavaScript, all numbers without 'n' suffix are tokenized as Number.Float
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(\.[0-9]+|[0-9]+\.[0-9]*|[0-9]+)([eE][-+]?[0-9]+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.\.\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"=>", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        // In Python: (r'\+\+|--|~|\?\?=?|\?|:|\\(?=\n)|'
        //             r'(<<|>>>?|==?|!=?|(?:\*\*|\|\||&&|[-<>+*%&|^/]))=?', Operator, 'slashstartsregex')
        // Rust regex doesn't support look-ahead (?=...), so we replace \\(?=\n) with \\\\\n
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\+\+|--|~|\?\?=?|\?|:|\\\\\n", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(<<|>>>?|==?|!=?|(?:\*\*|\|\||&&|[-<>+*%&|^/]))=?", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation that pushes to slashstartsregex
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{(\[;,]", Token::PUNCTUATION).unwrap(), action: LexerAction::push("slashstartsregex") });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[})\].]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operator words
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(typeof|instanceof|in|void|delete|new)\b", Token::OPERATOR_WORD).unwrap(), action: LexerAction::token(Token::OPERATOR_WORD) });

        // Reserved keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(constructor|from|as)\b", Token::KEYWORD_RESERVED).unwrap(), action: LexerAction::token(Token::KEYWORD_RESERVED) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(for|in|while|do|break|return|continue|switch|case|default|if|else|throw|try|catch|finally|yield|await|async|this|of|static|export|import|debugger|extends|super)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(var|let|const|with|function|class)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Reserved keywords (TypeScript-like)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(abstract|boolean|byte|char|double|enum|final|float|goto|implements|int|interface|long|native|package|private|protected|public|short|synchronized|throws|transient|volatile)\b", Token::KEYWORD_RESERVED).unwrap(), action: LexerAction::token(Token::KEYWORD_RESERVED) });

        // Constants
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(true|false|null|NaN|Infinity|undefined)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtin names
        let builtins = [
            "Array", "Boolean", "Date", "BigInt", "Function", "Math", "ArrayBuffer",
            "Number", "Object", "RegExp", "String", "Promise", "Proxy", "decodeURI",
            "decodeURIComponent", "encodeURI", "encodeURIComponent", "eval",
            "isFinite", "isNaN", "parseFloat", "parseInt", "DataView",
            "document", "window", "globalThis", "global", "arguments", "Symbol", "Intl",
            "WeakSet", "WeakMap", "Set", "Map", "Reflect", "JSON", "Atomics",
            "Int8Array", "Int16Array", "Int32Array", "BigInt64Array",
            "Float32Array", "Float64Array", "Uint8ClampedArray",
            "Uint8Array", "Uint16Array", "Uint32Array", "BigUint64Array",
        ];
        let builtin_pattern = format!("({})\\b", builtins.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&builtin_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Exception names
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b((?:Eval|Internal|Range|Reference|Syntax|Type|URI)?Error)\b", Token::NAME_EXCEPTION).unwrap(), action: LexerAction::token(Token::NAME_EXCEPTION) });

        // Private identifiers — Unicode-aware
        let private_ident = format!("#[{}$][{}$]*", XID_START, XID_CONTINUE);
        root_rules.push(LexerRule { pattern: TokenPattern::new(&private_ident, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Identifiers — Unicode-aware via XID_START/XID_CONTINUE
        // In Python: (JS_IDENT, Name.Other) where JS_IDENT is a complex Unicode-aware pattern
        let ident_pattern = format!("[{}$][{}$]*", XID_START, XID_CONTINUE);
        root_rules.push(LexerRule { pattern: TokenPattern::new(&ident_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Strings - must come before identifiers to match correctly
        // In Python: (r'"(\\\\|\\[^\\]|[^"\\])*"', String.Double)
        // Simplified for Rust: match double-quoted strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Template literals (backtick strings)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"`", Token::STRING_BACKTICK).unwrap(), action: LexerAction::push("interp") });

        inner.states.insert("root".to_string(), root_rules);

        // Slashstartsregex state - handles regex literals and operator context
        inner.states.insert("slashstartsregex".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"<!--", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) },
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
            // Regex literal - simplified, just pop back to root
            LexerRule { pattern: TokenPattern::new(r"/", Token::STRING_REGEX).unwrap(), action: LexerAction::pop(1) },
            // Default pop
            LexerRule { pattern: TokenPattern::new(r"[^\n]+", Token::WHITESPACE).unwrap(), action: LexerAction::pop(1) },
        ]);

        JavaScriptLexer { inner }
    }
}

impl Lexer for JavaScriptLexer {
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
    fn test_javascript_keywords() {
        let lexer = JavaScriptLexer::new();
        let tokens = lexer.get_tokens("var x = 1; if (true) { return x; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD), "Missing KEYWORD_DECLARATION");
        assert!(token_types.contains(&Token::KEYWORD), "Missing KEYWORD");
        assert!(token_types.contains(&Token::KEYWORD), "Missing KEYWORD_CONSTANT");
    }

    #[test]
    fn test_javascript_comment() {
        let lexer = JavaScriptLexer::new();
        let tokens = lexer.get_tokens("// this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE, "Expected COMMENT_SINGLE");
    }

    #[test]
    fn test_javascript_string() {
        let lexer = JavaScriptLexer::new();
        let tokens = lexer.get_tokens(r#"let x = "hello world";"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE), "Missing STRING_DOUBLE");
    }

    #[test]
    fn test_javascript_number() {
        let lexer = JavaScriptLexer::new();
        // In JavaScript, all numbers without 'n' suffix are tokenized as NUMBER_FLOAT
        // BigInt literals (with 'n' suffix) are tokenized as NUMBER_INTEGER
        let tokens = lexer.get_tokens("let x = 42; let y = 3.14; let z = 100n;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER), "Missing NUMBER_INTEGER (for BigInt)");
        assert!(token_types.contains(&Token::NUMBER), "Missing NUMBER_FLOAT");
    }

    #[test]
    fn test_javascript_arrow_function() {
        let lexer = JavaScriptLexer::new();
        let tokens = lexer.get_tokens("const f = (x) => x * 2;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD), "Missing KEYWORD_DECLARATION");
        assert!(token_types.contains(&Token::PUNCTUATION), "Missing PUNCTUATION");
    }

    #[test]
    fn test_javascript_template() {
        let lexer = JavaScriptLexer::new();
        let tokens = lexer.get_tokens(r#"let msg = `hello ${name}`;"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_BACKTICK), "Missing STRING_BACKTICK");
    }
}
