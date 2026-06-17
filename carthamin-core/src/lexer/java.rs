use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Java lexer supporting Java 17+ features.
pub struct JavaLexer {
    inner: RegexLexer,
}

impl JavaLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Java");
        inner.aliases = vec!["java"];
        inner.filenames = vec!["*.java"];
        inner.mimetypes = vec!["text/x-java"];

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

        // Numeric literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[oO][0-7_]+[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*\.[0-9_]+([eE][+-]?[0-9_]+)?[fFdD]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|\.\.|::", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]=?|[<>]=?|<<=?|>>=?|&=|&&|\|=|\|\||\^=|\!", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(abstract|assert|break|case|catch|class|const|continue|default|do|else|enum|exports|extends|final|finally|for|goto|if|implements|import|instanceof|interface|module|native|new|open|opens|package|private|protected|provides|public|requires|return|static|strictfp|super|switch|synchronized|this|throw|throws|to|transient|try|uses|var|volatile|while|yield)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(true|false|null)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtin types
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(boolean|byte|char|double|float|int|long|short|void)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Text blocks (triple-quoted strings)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""""[^"]*?""""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Characters
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        JavaLexer { inner }
    }
}

impl Lexer for JavaLexer {
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
    fn test_java_basic() {
        let lexer = JavaLexer::new();
        let tokens = lexer.get_tokens("public class Main { public static void main(String[] args) {} }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_java_comment() {
        let lexer = JavaLexer::new();
        let tokens = lexer.get_tokens("// this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_java_string() {
        let lexer = JavaLexer::new();
        let tokens = lexer.get_tokens(r#"System.out.println("hello");"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_java_number() {
        let lexer = JavaLexer::new();
        let tokens = lexer.get_tokens("int x = 42;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }
}
