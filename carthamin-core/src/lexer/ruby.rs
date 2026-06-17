use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Ruby lexer for Ruby source code.
pub struct RubyLexer {
    inner: RegexLexer,
}

impl RubyLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Ruby");
        inner.aliases = vec!["ruby", "rb", "duby"];
        inner.filenames = vec!["*.rb", "*.rbw", "Rakefile", "*.rake", "*.gemspec", "*.rbx", "*.duby", "Gemfile", "Vagrantfile"];
        inner.mimetypes = vec!["text/x-ruby", "application/x-ruby"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"`([^`\\]|\\.)*`", Token::STRING_BACKTICK).unwrap(), action: LexerAction::token(Token::STRING_BACKTICK) });

        // Symbols
        root_rules.push(LexerRule { pattern: TokenPattern::new(r":[a-zA-Z_][a-zA-Z0-9_]*[!?]?", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"&&|\|\||===|==|!=|<=|>=|<=>|<|<<|<>|>|>>|\?|\:|\+|-|\*|\/|%|\^|\|&|~|`|@|\$|\[|\]|\{|\}|\(|\)|;|,|\.|\.\.\.|=>|->|::", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "alias", "and", "begin", "break", "case", "class", "def", "defined?",
            "do", "else", "elsif", "end", "ensure", "for", "if", "in", "module",
            "next", "not", "or", "redo", "rescue", "retry", "return", "self",
            "super", "then", "undef", "unless", "until", "when", "while", "yield",
            "require", "require_relative", "include", "extend", "prepend", "import",
            "public", "private", "protected", "module_function", "const", "CONSTANT",
            "true", "false", "nil", "nil?", "self", "__FILE__", "__LINE__", "__dir__",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*[!?]?", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        RubyLexer { inner }
    }
}

impl Lexer for RubyLexer {
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
    fn test_ruby_keywords() {
        let lexer = RubyLexer::new();
        let tokens = lexer.get_tokens("class Hello; def world; puts 'hello'; end; end");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_ruby_comment() {
        let lexer = RubyLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
