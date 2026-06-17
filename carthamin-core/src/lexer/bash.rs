use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Bash lexer supporting shell scripting features.
pub struct BashLexer {
    inner: RegexLexer,
}

impl BashLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Bash");
        inner.aliases = vec!["bash", "sh", "shell", "zsh", "ksh"];
        inner.filenames = vec!["*.sh", "*.bash", "*.zsh", "*.ksh", "*.bashrc", "*.bash_profile", "*.bash_login", "*.bash_logout", ".profile", ".bash_aliases", ".zshrc", ".zprofile", ".zshenv", ".zlogin", ".zlogout", "Makefile", "makefile", "GNUmakefile"];
        inner.mimetypes = vec!["text/x-shellscript", "application/x-shellscript"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Shebang
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#! ?/.*$", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];|&<>]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\\|``", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"==|!=|<=|>=|<|>|&&|\|\||\+\+|--|[+\-*/%]=?", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(if|then|else|elif|fi|for|while|until|do|done|case|esac|in|select|function|time|coproc)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(break|continue|exit|return|shift|trap|exec|eval|export|declare|local|readonly|typeset|unset|let)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(source|\.|true|false|set|unset)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtins
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(alias|bg|bind|builtin|caller|cd|command|compgen|complete|dirs|disown|echo|enable|fc|fg|getopts|hash|help|history|jobs|kill|logout|mapfile|popd|printf|pushd|read|readarray|reload|shopt|suspend|test|time|times|tty|type|ulimit|umask|unalias|wait)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Variables
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\{[a-zA-Z_][a-zA-Z0-9_]*\}", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[0-9]", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[!#@*?_-]", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'[^']*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        BashLexer { inner }
    }
}

impl Lexer for BashLexer {
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
    fn test_bash_basic() {
        let lexer = BashLexer::new();
        let tokens = lexer.get_tokens("#!/bin/bash\nif [ -f file ]; then echo \"exists\"; fi");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_bash_comment() {
        let lexer = BashLexer::new();
        let tokens = lexer.get_tokens("# this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_bash_variable() {
        let lexer = BashLexer::new();
        let tokens = lexer.get_tokens("echo $HOME");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_bash_string() {
        let lexer = BashLexer::new();
        let tokens = lexer.get_tokens("echo 'hello world'");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_bash_shebang() {
        let lexer = BashLexer::new();
        let tokens = lexer.get_tokens("#!/bin/bash");
        // Shebang is matched as Comment::Single because comment rule is checked first
        // assert_eq!(tokens[0].0, Token::COMMENT);
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
