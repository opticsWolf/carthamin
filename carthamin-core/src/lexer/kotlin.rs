use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Kotlin lexer for Kotlin source code.
/// Ported from pygments.lexers.jvm.KotlinLexer.
pub struct KotlinLexer {
    inner: RegexLexer,
}

impl KotlinLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Kotlin");
        inner.aliases = vec!["kotlin", "kt", "kts"];
        inner.filenames = vec!["*.kt", "*.kts"];
        inner.mimetypes = vec!["text/x-kotlin"];

        // Kotlin identifier patterns (simplified ASCII version)
        let kt_name = r"@?[_a-zA-Z][_a-zA-Z0-9]*";
        let kt_id = format!(r"(?:(?:{})|(?:`[^`]*`))", kt_name);

        // Modifiers
        let modifiers = r"actual|abstract|annotation|companion|const|crossinline|data|enum|expect|external|final|infix|inline|inner|internal|lateinit|noinline|open|operator|override|private|protected|public|sealed|suspend|tailrec|value";

        // Built-in types
        let nullable_types = r"(?:Boolean|Byte|Char|Double|Float|Int|Long|Short|String|Any|Unit)\?";
        let types = r"(?:Boolean|Byte|Char|Double|Float|Int|Long|Short|String|Any|Unit)";

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace
        root_rules.push(LexerRule::token(r"\s+", Token::WHITESPACE).unwrap());
        root_rules.push(LexerRule::token(r"\\$", Token::STRING_ESCAPE).unwrap());

        // Comments
        root_rules.push(LexerRule::token(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap());
        root_rules.push(LexerRule::token(r"^#![^\n]*", Token::COMMENT_SINGLE).unwrap()); // shebang
        root_rules.push(LexerRule::token(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap());

        // Keywords
        root_rules.push(LexerRule::token(r"as\?", Token::KEYWORD).unwrap());
        root_rules.push(LexerRule::token(r"(as|break|by|catch|constructor|continue|do|dynamic|else|finally|get|for|if|init|[!]*in|[!]*is|out|reified|return|set|super|this|throw|try|typealias|typeof|vararg|when|where|while)\b", Token::KEYWORD).unwrap());

        // it (built-in)
        root_rules.push(LexerRule::token(r"it\b", Token::NAME_BUILTIN).unwrap());

        // Built-in types (nullable first, then non-nullable)
        root_rules.push(LexerRule::token(nullable_types, Token::KEYWORD_TYPE).unwrap());
        root_rules.push(LexerRule::token(&format!("{}\\b", types), Token::KEYWORD_TYPE).unwrap());

        // Constants
        root_rules.push(LexerRule::token(r"(true|false|null)\b", Token::KEYWORD_CONSTANT).unwrap());

        // Imports
        root_rules.push(LexerRule::token(r"(package|import)(\s+)(\S+)", Token::KEYWORD).unwrap());

        // Dot access
        root_rules.push(LexerRule::token(r"(\?\.)((?:[^\W\d]|\$)[\w\$]*)", Token::OPERATOR).unwrap());
        root_rules.push(LexerRule::token(r"(\.)((?:[^\W\d]|\$)[\w\$]*)", Token::PUNCTUATION).unwrap());

        // Annotations
        root_rules.push(LexerRule::token(r"@[^\W\d][\w.]*", Token::NAME_DECORATOR).unwrap());

        // Labels
        root_rules.push(LexerRule::token(r"[^\W\d][\w.]+@", Token::NAME_DECORATOR).unwrap());

        // Object expression
        root_rules.push(LexerRule::push(r"(object)(\s+)(:)(\s+)", Token::KEYWORD, "class").unwrap());

        // Class/interface/object declarations (including fun interface)
        root_rules.push(LexerRule::push(&format!(r"((?:(?:{}|fun)\s+)*)(class|interface|object)(\s+)", modifiers), Token::KEYWORD, "class").unwrap());

        // Variables
        root_rules.push(LexerRule::push(r"(var|val)(\s+)(\()", Token::KEYWORD, "destructuring_assignment").unwrap());
        root_rules.push(LexerRule::push(&format!(r"((?:(?:{})\s+)*)(var|val)(\s+)", modifiers), Token::KEYWORD, "variable").unwrap());

        // Functions
        root_rules.push(LexerRule::push(&format!(r"((?:(?:{})\s+)*)(fun)(\s+)", modifiers), Token::KEYWORD, "function").unwrap());

        // Operators
        root_rules.push(LexerRule::token(r"::|!!|\?[:.]", Token::OPERATOR).unwrap());
        root_rules.push(LexerRule::token(r"[~^*!%&\[\]<>|+=/?-]", Token::OPERATOR).unwrap());

        // Punctuation
        root_rules.push(LexerRule::token(r"[{}();:.,]", Token::PUNCTUATION).unwrap());

        // Strings - triple-quoted
        root_rules.push(LexerRule::push(r#""""#, Token::STRING, "multiline_string").unwrap());

        // Strings - double-quoted
        root_rules.push(LexerRule::push(r#"""#, Token::STRING, "string").unwrap());

        // Character literals
        root_rules.push(LexerRule::token(r"'\\.'|'[^\\]'", Token::STRING_CHAR).unwrap());

        // Numbers
        root_rules.push(LexerRule::token(r"[0-9](\.[0-9]*)?([eE][+-][0-9]+)?[flFL]?|0[xX][0-9a-fA-F]+[Ll]?", Token::NUMBER).unwrap());

        // Identifiers (with optional nullable ? suffix)
        root_rules.push(LexerRule::token(&format!(r"{}(?:\?[^.])?", kt_id), Token::NAME).unwrap());

        inner.add_state("root", root_rules);

        // Class state — handles class name and enters generic state on "<"
        // Does NOT pop after matching the name; stays to handle optional generics.
        // Falls back to pop on unrecognized input.
        inner.add_state("class", vec![
            LexerRule::push(r"<", Token::OPERATOR, "generic").unwrap(),
            LexerRule { pattern: TokenPattern::new(&kt_id, Token::NAME_CLASS).unwrap(), action: LexerAction::token(Token::NAME_CLASS) },
            LexerRule { pattern: TokenPattern::new(r".", Token::TEXT).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Variable state
        inner.add_state("variable", vec![
            LexerRule { pattern: TokenPattern::new(&kt_id, Token::NAME_VARIABLE).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Destructuring assignment state
        inner.add_state("destructuring_assignment", vec![
            LexerRule::token(r",", Token::PUNCTUATION).unwrap(),
            LexerRule::token(r"\s+", Token::WHITESPACE).unwrap(),
            LexerRule { pattern: TokenPattern::new(&kt_id, Token::NAME_VARIABLE).unwrap(), action: LexerAction::token(Token::NAME_VARIABLE) },
            LexerRule::token(&format!(r"(:)(\s+)({})", kt_id), Token::PUNCTUATION).unwrap(),
            LexerRule::push(r"<", Token::OPERATOR, "generic").unwrap(),
            LexerRule { pattern: TokenPattern::new(r"\)", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Function state (with extension function support)
        inner.add_state("function", vec![
            LexerRule::push(r"<", Token::OPERATOR, "generic").unwrap(),
            LexerRule { pattern: TokenPattern::new(&format!(r"{}\.{}", kt_id, kt_id), Token::NAME_FUNCTION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(&kt_id, Token::NAME_FUNCTION).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Generic state — supports nested generics via recursive push on "<"
        inner.add_state("generic", vec![
            LexerRule::push(r"<", Token::OPERATOR, "generic").unwrap(),
            LexerRule { pattern: TokenPattern::new(r">", Token::OPERATOR).unwrap(), action: LexerAction::pop(1) },
            LexerRule::token(r":", Token::PUNCTUATION).unwrap(),
            LexerRule::token(r"(reified|out|in)\b", Token::KEYWORD_DECLARATION).unwrap(),
            LexerRule::token(r",", Token::PUNCTUATION).unwrap(),
            LexerRule::token(r"\s+", Token::WHITESPACE).unwrap(),
            LexerRule::token(&kt_id, Token::NAME).unwrap(),
        ]);

        // Modifier state
        inner.add_state("modifiers", vec![
            LexerRule::token(r"\w+", Token::KEYWORD_DECLARATION).unwrap(),
            LexerRule::token(r"\s+", Token::WHITESPACE).unwrap(),
        ]);

        // String common rules (shared by string and multiline_string)
        // IMPORTANT: In a raw string r#"..."#, \" is a literal backslash+quote.
        // To match any char except ", $, or \, use: r#"[^"$\\]+"#
        let string_common: Vec<LexerRule> = vec![
            LexerRule::token(r"\\\\", Token::STRING_ESCAPE).unwrap(),
            LexerRule::token(r#"\\"#, Token::STRING_ESCAPE).unwrap(),
            LexerRule::token(r"\\", Token::STRING_ESCAPE).unwrap(),
            LexerRule::push(r"\$\{", Token::STRING_INTERPOL, "interpolation").unwrap(),
            LexerRule::token(r"\$\w+", Token::STRING_INTERPOL).unwrap(),
            // Matches any chars except ", $, and backslash
            LexerRule::token(r#"[^"$\\]+"#, Token::STRING).unwrap(),
        ];

        // String state (double-quoted)
        inner.add_state("string", {
            let mut rules = vec![
                LexerRule { pattern: TokenPattern::new(r#"""#, Token::STRING).unwrap(), action: LexerAction::pop(1) },
            ];
            rules.extend(string_common.clone());
            rules
        });

        // Multiline string state (triple-quoted)
        inner.add_state("multiline_string", {
            let mut rules = vec![
                LexerRule { pattern: TokenPattern::new(r#""""#, Token::STRING).unwrap(), action: LexerAction::pop(1) },
                LexerRule::token(r#"""#, Token::STRING).unwrap(),
            ];
            rules.extend(string_common);
            rules
        });

        // Interpolation state
        inner.add_state("interpolation", vec![
            LexerRule::token(r#"""#, Token::STRING).unwrap(),
            LexerRule::push(r"\$\{", Token::STRING_INTERPOL, "interpolation").unwrap(),
            LexerRule::push(r"\{", Token::PUNCTUATION, "scope").unwrap(),
            LexerRule { pattern: TokenPattern::new(r"\}", Token::STRING_INTERPOL).unwrap(), action: LexerAction::pop(1) },
        ]);

        // Scope state (for nested braces in interpolation)
        inner.add_state("scope", vec![
            LexerRule::push(r"\{", Token::PUNCTUATION, "scope").unwrap(),
            LexerRule { pattern: TokenPattern::new(r"\}", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
        ]);

        KotlinLexer { inner }
    }
}

impl Lexer for KotlinLexer {
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
    fn test_kotlin_keywords() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("fun main() { println(\"Hello\") }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_kotlin_comment() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("// comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_kotlin_string() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("\"hello\"");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_kotlin_class() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("class MyClass");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME_CLASS));
    }

    #[test]
    fn test_kotlin_variable() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("val x = 1");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_kotlin_multiline_string() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("\"\"\"hello world\"\"\"");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_kotlin_string_interpolation() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("\"hello ${name}\"");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_INTERPOL));
    }

    #[test]
    fn test_kotlin_annotation() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("@Deprecated fun foo() {}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_DECORATOR));
    }

    #[test]
    fn test_kotlin_data_class() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("data class User(val name: String)");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_kotlin_when() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("when (x) { 1 -> \"one\" else -> \"other\" }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_kotlin_fun_interface() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("fun interface MyFunc");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME_CLASS));
    }

    #[test]
    fn test_kotlin_extension_function() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("fun String.trimWhitespace()");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME_FUNCTION));
    }

    #[test]
    fn test_kotlin_nullable_type() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("val x: String? = null");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD_TYPE));
    }

    #[test]
    fn test_kotlin_shebang() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("#!/usr/bin/env kotlin\nprintln(\"hello\")");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::COMMENT_SINGLE));
    }

    #[test]
    fn test_kotlin_nested_generics() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("fun <T> foo(): List<T>");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_kotlin_generic_modifiers() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("class Foo<out T : Comparable<T>>");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD_DECLARATION));
    }

    #[test]
    fn test_kotlin_string_with_escape() {
        let lexer = KotlinLexer::new();
        let tokens = lexer.get_tokens("\"hello\\nworld\"");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_ESCAPE));
    }
}
