use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Swift lexer for Swift source code.
pub struct SwiftLexer {
    inner: RegexLexer,
}

impl SwiftLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Swift");
        inner.aliases = vec!["swift"];
        inner.filenames = vec!["*.swift"];
        inner.mimetypes = vec!["text/x-swift"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Raw strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"#""([^#\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // String interpolation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"\(([^)]+)\)"#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|=>|===|!==|==|!=|<=|>=|&&|\|\||\?\?|\?\.", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%&|^<>]=?|~|!|=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "import", "extension", "struct", "class", "protocol", "enum",
            "func", "init", "deinit", "return", "if", "else", "switch",
            "case", "for", "while", "repeat", "in", "break", "continue",
            "do", "try", "catch", "throw", "defer", "guard", "where",
            "as", "is", "typealias", "associatedtype", "subscript",
            "get", "set", "willSet", "didSet", "override", "mutating",
            "nonmutating", "class", "static", "final", "open", "public",
            "internal", "fileprivate", "private", "lazy", "weak", "unowned",
            "required", "convenience", "optional", "inout", "var", "let",
            "true", "false", "nil", "self", "super", "init", "deinit",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Type keywords
        let type_keywords = [
            "Int", "Int8", "Int16", "Int32", "Int64", "UInt", "UInt8",
            "UInt16", "UInt32", "UInt64", "Float", "Double", "Bool",
            "String", "Character", "Array", "Dictionary", "Set", "Optional",
            "Never", "Any", "AnyObject",
        ];
        let type_pattern = format!(r"\b({})\b", type_keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        SwiftLexer { inner }
    }
}

impl Lexer for SwiftLexer {
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
    fn test_swift_keywords() {
        let lexer = SwiftLexer::new();
        let tokens = lexer.get_tokens("class Hello { func main() { } }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_swift_comment() {
        let lexer = SwiftLexer::new();
        let tokens = lexer.get_tokens("// comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
