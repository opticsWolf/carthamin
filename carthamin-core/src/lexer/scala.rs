use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Scala lexer for Scala source code.
pub struct ScalaLexer {
    inner: RegexLexer,
}

impl ScalaLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Scala");
        inner.aliases = vec!["scala"];
        inner.filenames = vec!["*.scala"];
        inner.mimetypes = vec!["text/x-scala"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_CHAR).unwrap(), action: LexerAction::token(Token::STRING_CHAR) });

        // Raw triple-quoted strings — FIXED: use r## to embed quotes, single backslashes
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##""""".*?"""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?[fFdDlL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators — FIXED: use single backslashes in raw string
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|=>|<:|<%|>:|&&|!=|==|<=|>=|<|>|!|~|\||\?|:|=|\+|-|\*|/|%|&|\^", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};:,.\@\#]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "abstract", "case", "class", "def", "do", "else", "enum", "export",
            "extends", "final", "finally", "for", "if", "implicit",
            "lazy", "match", "new", "null", "override", "package",
            "private", "protected", "return", "sealed", "super", "this", "throw",
            "try", "type", "val", "var", "while", "with", "yield",
            "catch", "clone", "const", "false", "for", "fun", "function",
            "import", "infix", "inline", "inner", "interface", "internal", "is",
            "object", "operator", "out", "public", "reified", "sealed",
            "super", "synchronized", "tailrec", "throws", "transient", "true",
            "typealias", "value", "vararg", "when", "where",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Type keywords
        let type_keywords = [
            "Any", "Array", "Boolean", "Byte", "Char", "Double",
            "Float", "Int", "Long", "Nothing", "Short", "String", "Unit",
        ];
        let type_pattern = format!(r"\b({})\b", type_keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        ScalaLexer { inner }
    }
}

impl Lexer for ScalaLexer {
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
    fn test_scala_keywords() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("class Hello { def main(args: Array[String]) { } }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_scala_comment() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("// comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
