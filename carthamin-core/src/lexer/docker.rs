use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Docker lexer for Dockerfile syntax.
pub struct DockerLexer {
    inner: RegexLexer,
}

impl DockerLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Docker");
        inner.aliases = vec!["docker", "dockerfile"];
        inner.filenames = vec!["Dockerfile", "*.dockerfile"];
        inner.mimetypes = vec!["text/x-dockerfile"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Dockerfile directives (uppercase keywords)
        let directives = [
            "FROM", "MAINTAINER", "RUN", "CMD", "LABEL", "EXPOSE", "ENV",
            "ADD", "COPY", "ENTRYPOINT", "USER", "WORKDIR", "VOLUME",
            "STOPSIGNAL", "SHELL", "ONBUILD", "ARG", "HEALTHCHECK",
        ];
        let directive_pattern = format!(r"^\s*({})\b", directives.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&directive_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Variables
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Images and paths
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z0-9_./:-]+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        DockerLexer { inner }
    }
}

impl Lexer for DockerLexer {
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
    fn test_docker_directives() {
        let lexer = DockerLexer::new();
        let tokens = lexer.get_tokens("FROM ubuntu:20.04\nRUN apt-get update");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_docker_comment() {
        let lexer = DockerLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
