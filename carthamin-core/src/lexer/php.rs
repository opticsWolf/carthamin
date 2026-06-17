use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// PHP lexer for PHP source code.
/// Ported from pygments.lexers.php.PhpLexer.
pub struct PhpLexer {
    inner: RegexLexer,
}

impl PhpLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("PHP");
        inner.aliases = vec!["php", "php3", "php4", "php5"];
        inner.filenames = vec!["*.php", "*.php[345]", "*.inc"];
        inner.mimetypes = vec!["text/x-php"];

        // PHP identifier patterns (simplified ASCII)
        let ident_inner = r"(?:[\\_a-zA-Z]|[^\x00-\x7f])(?:[\\\w]|[^\x00-\x7f])*";
        let ident_nons = r"(?:[_a-zA-Z]|[^\x00-\x7f])(?:\w|[^\x00-\x7f])*";

        // PHP reserved keywords
        let php_keywords = r"and|E_PARSE|old_function|E_ERROR|or|as|E_WARNING|parent|eval|PHP_OS|break|exit|case|extends|PHP_VERSION|cfunction|FALSE|print|for|require|continue|foreach|require_once|declare|return|default|static|do|switch|die|stdClass|echo|else|TRUE|elseif|var|empty|if|xor|enddeclare|include|virtual|endfor|include_once|while|endforeach|global|endif|list|endswitch|new|endwhile|not|array|E_ALL|NULL|final|php_user_filter|interface|implements|public|private|protected|abstract|clone|try|catch|throw|this|use|namespace|trait|yield|finally|match|readonly";

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // PHP opening tag
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<\?(php)?", Token::COMMENT_PREPROC).unwrap(), action: LexerAction::push("php") });

        // Non-PHP content
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[^<]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        inner.add_state("root", root_rules);

        // PHP state — built via helper so it can be reused in attributeparams
        let php_rules = Self::build_php_rules(&ident_inner, &ident_nons, php_keywords);
        inner.add_state("php", php_rules);

        // Variable variable state
        inner.add_state("variablevariable", vec![
            LexerRule { pattern: TokenPattern::new(r"\x7d", Token::NAME_VARIABLE).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r".", Token::NAME_VARIABLE).unwrap(), action: LexerAction::token(Token::NAME_VARIABLE) },
        ]);

        // Class name state
        inner.add_state("classname", vec![
            LexerRule { pattern: TokenPattern::new(&ident_inner, Token::NAME_CLASS).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Function name state
        inner.add_state("functionname", vec![
            LexerRule { pattern: TokenPattern::new(r"__[a-zA-Z_]\w*", Token::NAME_FUNCTION_MAGIC).unwrap(), action: LexerAction::token(Token::NAME_FUNCTION_MAGIC) },
            LexerRule { pattern: TokenPattern::new(&ident_inner, Token::NAME_FUNCTION).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Function args state — handles the opening paren as PUNCTUATION
        // This replaces the look-ahead (?=\() approach since Rust regex doesn't support it.
        inner.add_state("function_args", vec![
            LexerRule { pattern: TokenPattern::new(r"\(", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
        ]);

        // String state (for double-quoted strings)
        inner.add_state("string", vec![
            LexerRule { pattern: TokenPattern::new(r"\x22", Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[^\x7b\x24\x22\x5c]+", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) },
            LexerRule { pattern: TokenPattern::new(r"\\\\", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r"\\[nrt\x22\x24\\]|[0-7]{1,3}|x[0-9a-f]{1,2}", Token::STRING_ESCAPE).unwrap(), action: LexerAction::token(Token::STRING_ESCAPE) },
            LexerRule { pattern: TokenPattern::new(r"\x24[a-zA-Z_]\w*(\[\S+?\]|->[a-zA-Z_]\w*)?", Token::STRING_INTERPOL).unwrap(), action: LexerAction::token(Token::STRING_INTERPOL) },
            LexerRule { pattern: TokenPattern::new(r"(\x7b)(\x24.*?)(\x7d)", Token::STRING_INTERPOL).unwrap(), action: LexerAction::token(Token::STRING_INTERPOL) },
            LexerRule { pattern: TokenPattern::new(r"(\x24\x7b)(\S+)(\x7d)", Token::STRING_INTERPOL).unwrap(), action: LexerAction::token(Token::STRING_INTERPOL) },
            LexerRule { pattern: TokenPattern::new(r"[\x24\x7b\x5c]", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) },
        ]);

        // Heredoc state — matches content until closing delimiter
        // Rust regex lacks backreferences, so we match any identifier at start of line
        // as a potential closing delimiter. This is a simplification of the Pygments
        // backreference approach.
        inner.add_state("heredoc", vec![
            LexerRule { pattern: TokenPattern::new(r"[A-Za-z_]\w*\s*;\n", Token::STRING).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[A-Za-z_]\w*\n", Token::STRING).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r".", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) },
        ]);

        // Attribute state
        inner.add_state("attribute", vec![
            LexerRule { pattern: TokenPattern::new(r"\x5d", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"\x28", Token::PUNCTUATION).unwrap(), action: LexerAction::push("attributeparams") },
            LexerRule { pattern: TokenPattern::new(&ident_inner, Token::NAME_DECORATOR).unwrap(), action: LexerAction::token(Token::NAME_DECORATOR) },
            // Include php rules for comments, whitespace, etc.
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"#.*?\n", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"//.*?\n", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
            LexerRule { pattern: TokenPattern::new(r"/\*\*.*?\*/", Token::STRING_DOC).unwrap(), action: LexerAction::token(Token::STRING_DOC) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
            LexerRule { pattern: TokenPattern::new(r"[\[\]{}();,]+", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) },
        ]);

        // Attribute params state — includes php rules for processing contents
        // so that #[MyAttr("String", 123)] is handled correctly.
        inner.add_state("attributeparams", {
            let mut rules = vec![
                LexerRule { pattern: TokenPattern::new(r"\x29", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            ];
            // Include most php rules for processing attribute arguments
            let ident_inner_local = ident_inner.clone();
            let ident_nons_local = ident_nons.clone();
            let php_keywords_local = php_keywords.clone();
            let php_include = Self::build_php_rules_include(&ident_inner_local, &ident_nons_local, php_keywords_local);
            rules.extend(php_include);
            rules
        });

        PhpLexer { inner }
    }

    /// Build the main PHP state rules.
    fn build_php_rules(ident_inner: &str, ident_nons: &str, php_keywords: &str) -> Vec<LexerRule> {
        let mut rules: Vec<LexerRule> = Vec::new();

        // PHP closing tag
        rules.push(LexerRule { pattern: TokenPattern::new(r"\?>", Token::COMMENT_PREPROC).unwrap(), action: LexerAction::pop(1) });

        // Heredoc/Nowdoc opener — push to heredoc state
        // FIXED: Use single backslashes in raw string for regex escapes.
        // Pygments: (r'(<<<)([\'"]?)(' + _ident_nons + r')(\2\n.*?\n\s*)(\3)(;?)(\n)')
        // Rust regex lacks backreferences, so we only match the opener and push to heredoc state.
        rules.push(LexerRule { pattern: TokenPattern::new(r"<<<[\x27\x22]?[_a-zA-Z][\w]*(\s|\n)", Token::STRING).unwrap(), action: LexerAction::push("heredoc") });

        // Whitespace
        rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Attributes (#[)
        rules.push(LexerRule { pattern: TokenPattern::new(r"#\x5b", Token::PUNCTUATION).unwrap(), action: LexerAction::push("attribute") });

        // Comments
        rules.push(LexerRule { pattern: TokenPattern::new(r"#.*?\n", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"//.*?\n", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Doc comments
        rules.push(LexerRule { pattern: TokenPattern::new(r"/\*\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"/\*\*.*?\*/", Token::STRING_DOC).unwrap(), action: LexerAction::token(Token::STRING_DOC) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Member access
        rules.push(LexerRule { pattern: TokenPattern::new(&format!(r"(->|::)(\s*)({})", ident_nons), Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Operators
        rules.push(LexerRule { pattern: TokenPattern::new(r"[~!%^&*+=|:.<>/@-]+", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"\?", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        rules.push(LexerRule { pattern: TokenPattern::new(r"[\[\]{}();,]+", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // New class
        rules.push(LexerRule { pattern: TokenPattern::new(r"(new)(\s+)(class)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Class declaration
        rules.push(LexerRule { pattern: TokenPattern::new(r"(class)(\s+)", Token::KEYWORD).unwrap(), action: LexerAction::push("classname") });

        // Function declaration — push to function_args to handle ( as PUNCTUATION
        // Pygments uses look-ahead (?=\() but Rust regex doesn't support it.
        rules.push(LexerRule { pattern: TokenPattern::new(r"(function)(\s*)", Token::KEYWORD).unwrap(), action: LexerAction::push("function_args") });
        rules.push(LexerRule { pattern: TokenPattern::new(r"(function)(\s+)(&?)(\s*)", Token::KEYWORD).unwrap(), action: LexerAction::push("functionname") });

        // Const declaration
        rules.push(LexerRule { pattern: TokenPattern::new(&format!(r"(const)(\s+)({})", ident_inner), Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Reserved keywords
        rules.push(LexerRule { pattern: TokenPattern::new(&format!(r"({})\b", php_keywords), Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Constants
        rules.push(LexerRule { pattern: TokenPattern::new(r"(true|false|null)\b", Token::KEYWORD_CONSTANT).unwrap(), action: LexerAction::token(Token::KEYWORD_CONSTANT) });

        // Magic constants
        rules.push(LexerRule { pattern: TokenPattern::new(r"__LINE__|__FILE__|__DIR__|__FUNCTION__|__CLASS__|__METHOD__|__NAMESPACE__|__TRAIT__|__PROPERTY__", Token::NAME_CONSTANT).unwrap(), action: LexerAction::token(Token::NAME_CONSTANT) });

        // Variable variables
        rules.push(LexerRule { pattern: TokenPattern::new(r"\$\x7b", Token::NAME_VARIABLE).unwrap(), action: LexerAction::push("variablevariable") });

        // Variables
        rules.push(LexerRule { pattern: TokenPattern::new(&format!(r"\$+{}", ident_inner), Token::NAME_VARIABLE).unwrap(), action: LexerAction::token(Token::NAME_VARIABLE) });

        // Other identifiers
        rules.push(LexerRule { pattern: TokenPattern::new(ident_inner, Token::NAME_OTHER).unwrap(), action: LexerAction::token(Token::NAME_OTHER) });

        // Numbers
        rules.push(LexerRule { pattern: TokenPattern::new(r"(\d+\.\d*|\d*\.\d+)(e[+-]?\d+)?", Token::NUMBER_FLOAT).unwrap(), action: LexerAction::token(Token::NUMBER_FLOAT) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"\d+e[+-]?\d+", Token::NUMBER_FLOAT).unwrap(), action: LexerAction::token(Token::NUMBER_FLOAT) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"0[0-7]+", Token::NUMBER_OCT).unwrap(), action: LexerAction::token(Token::NUMBER_OCT) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"0x[a-f0-9]+", Token::NUMBER_HEX).unwrap(), action: LexerAction::token(Token::NUMBER_HEX) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"\d+", Token::NUMBER_INTEGER).unwrap(), action: LexerAction::token(Token::NUMBER_INTEGER) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"0b[01]+", Token::NUMBER_BIN).unwrap(), action: LexerAction::token(Token::NUMBER_BIN) });

        // Single-quoted strings
        rules.push(LexerRule { pattern: TokenPattern::new(r"'[^']*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Backtick strings
        rules.push(LexerRule { pattern: TokenPattern::new(r"`[^`]*`", Token::STRING_BACKTICK).unwrap(), action: LexerAction::token(Token::STRING_BACKTICK) });

        // Double-quoted strings
        rules.push(LexerRule { pattern: TokenPattern::new(r"\x22", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("string") });

        rules
    }

    /// Build a subset of PHP rules for inclusion in other states (attributeparams, etc.)
    fn build_php_rules_include(ident_inner: &str, _ident_nons: &str, php_keywords: &str) -> Vec<LexerRule> {
        let mut rules: Vec<LexerRule> = Vec::new();

        // Whitespace
        rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        rules.push(LexerRule { pattern: TokenPattern::new(r"#.*?\n", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"//.*?\n", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"/\*\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"/\*\*.*?\*/", Token::STRING_DOC).unwrap(), action: LexerAction::token(Token::STRING_DOC) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Operators
        rules.push(LexerRule { pattern: TokenPattern::new(r"[~!%^&*+=|:.<>/@-]+", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        rules.push(LexerRule { pattern: TokenPattern::new(r"[\[\]{}();,]+", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        rules.push(LexerRule { pattern: TokenPattern::new(&format!(r"({})\b", php_keywords), Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"(true|false|null)\b", Token::KEYWORD_CONSTANT).unwrap(), action: LexerAction::token(Token::KEYWORD_CONSTANT) });

        // Variables
        rules.push(LexerRule { pattern: TokenPattern::new(&format!(r"\$+{}", ident_inner), Token::NAME_VARIABLE).unwrap(), action: LexerAction::token(Token::NAME_VARIABLE) });

        // Identifiers
        rules.push(LexerRule { pattern: TokenPattern::new(ident_inner, Token::NAME_OTHER).unwrap(), action: LexerAction::token(Token::NAME_OTHER) });

        // Numbers
        rules.push(LexerRule { pattern: TokenPattern::new(r"\d+e[+-]?\d+", Token::NUMBER_FLOAT).unwrap(), action: LexerAction::token(Token::NUMBER_FLOAT) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"(\d+\.\d*|\d*\.\d+)(e[+-]?\d+)?", Token::NUMBER_FLOAT).unwrap(), action: LexerAction::token(Token::NUMBER_FLOAT) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"0x[a-f0-9]+", Token::NUMBER_HEX).unwrap(), action: LexerAction::token(Token::NUMBER_HEX) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"\d+", Token::NUMBER_INTEGER).unwrap(), action: LexerAction::token(Token::NUMBER_INTEGER) });

        // Strings
        rules.push(LexerRule { pattern: TokenPattern::new(r"'[^']*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });
        rules.push(LexerRule { pattern: TokenPattern::new(r"\x22", Token::STRING_DOUBLE).unwrap(), action: LexerAction::push("string") });

        rules
    }
}

impl Lexer for PhpLexer {
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
    fn test_php_opening_tag() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php echo 'hello'; ?>");
        assert_eq!(tokens[0].0, Token::COMMENT_PREPROC);
    }

    #[test]
    fn test_php_comment() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php // comment\n");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::COMMENT_SINGLE));
    }

    #[test]
    fn test_php_string() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php echo 'hello world';");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_SINGLE));
    }

    #[test]
    fn test_php_variable() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php $variable = 123;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_VARIABLE));
    }

    #[test]
    fn test_php_class() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php class MyClass {}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME_CLASS));
    }

    #[test]
    fn test_php_function() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php function myFunc() {}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_php_double_string() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php echo \"hello world\";");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_php_string_interpolation() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php echo \"hello {$name}\";");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_INTERPOL));
    }

    #[test]
    fn test_php_magic_constant() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php echo __FILE__;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_CONSTANT));
    }

    #[test]
    fn test_php_attribute() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php #[Attribute] function foo() {}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_DECORATOR));
    }

    #[test]
    fn test_php_function_paren() {
        // The function_args state should tokenize ( as PUNCTUATION
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php function foo() {}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::PUNCTUATION));
    }

    #[test]
    fn test_php_attribute_with_params() {
        // Attribute params should process contents like normal PHP
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php #[MyAttr(\"value\")] function foo() {}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_DECORATOR));
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_php_heredoc() {
        let lexer = PhpLexer::new();
        let tokens = lexer.get_tokens("<?php $x = <<<EOF\nhello world\nEOF;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }
}
