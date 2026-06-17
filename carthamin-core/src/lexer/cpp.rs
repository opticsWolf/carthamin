use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// C/C++ lexer supporting both C and C++ features.
pub struct CppLexer {
    inner: RegexLexer,
}

impl CppLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("C++");
        inner.aliases = vec!["cpp", "c++", "hpp", "cxx", "cc"];
        inner.filenames = vec!["*.cpp", "*.hpp", "*.cxx", "*.h", "*.cc", "*.hh"];
        inner.mimetypes = vec!["text/x-c++src", "text/x-c++hdr", "text/x-csrc", "text/x-chdr"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Include comments and whitespace
        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Preprocessor directives
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"^#include\s*[<"].*?[>"]"#, Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#[a-zA-Z_]\w*\b", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#.*", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Numeric literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[oO]?[0-7_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*\.[0-9_]+([eE][+-]?[0-9_]+)?[fF]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9_]+\.[0-9_]*([eE][+-]?[0-9_]+)?[fF]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|\.\.|::", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]=?|[<>]=?|<<=?|>>=?|&=|&&|\|=|\|\||\^=|\!|~", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(alignas|alignof|asm|attribute|bool|catch|class|concept|const_cast|constexpr|consteval|constinit|decltype|delete|dynamic_cast|explicit|export|friend|goto|ifconsteval|mutable|namespace|new|noexcept|nullptr|operator|override|requires|static_assert|static_cast|template|this|thread_local|throw|true|false|typeid|typename|try|using|virtual|co_await|co_return|co_yield)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtin types
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(nullptr|NULL|stdin|stdout|stderr|true|false)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Characters
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        CppLexer { inner }
    }
}

impl Lexer for CppLexer {
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

/// C lexer (subset of C++).
pub struct CLexer {
    inner: RegexLexer,
}

impl CLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("C");
        inner.aliases = vec!["c", "h"];
        inner.filenames = vec!["*.c", "*.h"];
        inner.mimetypes = vec!["text/x-csrc", "text/x-chdr"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Preprocessor directives
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"^#include\s*[<"].*?[>"]"#, Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#[a-zA-Z_]\w*\b", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#.*", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Numeric literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*\.[0-9_]+([eE][+-]?[0-9_]+)?[fF]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]=?|[<>]=?|<<=?|>>=?|&=|&&|\|=|\|\||\^=|\!", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Characters
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        CLexer { inner }
    }
}

impl Lexer for CLexer {
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
    fn test_cpp_basic() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("int main() { return 0; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_cpp_preprocessor() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("#include <iostream>");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }

    #[test]
    fn test_cpp_comment() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("// this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_cpp_string() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens(r#"std::cout << "hello";"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_c_basic() {
        let lexer = CLexer::new();
        let tokens = lexer.get_tokens("#include <stdio.h>\nint main() { return 0; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::COMMENT));
        assert!(token_types.contains(&Token::KEYWORD));
    }
}
