use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Makefile lexer for Makefile syntax.
pub struct MakefileLexer {
    inner: RegexLexer,
}

impl MakefileLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Makefile");
        inner.aliases = vec!["makefile", "make"];
        inner.filenames = vec!["Makefile", "makefile", "*.mk"];
        inner.mimetypes = vec!["text/x-makefile"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Comments (before whitespace so # at start of line is caught)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Whitespace and tabs
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Variables
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\([^\)]*\)", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\{[^\}]*\}", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\w+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"::=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r":=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\?=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\+=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"!=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Targets (name followed by colon)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z0-9_./%-]+:", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Phony targets and special targets
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.PHONY\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.SUFFIXES\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.DEFAULT\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[;@%|]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Functions
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(word|words|firstword|lastword|wordlist|filter|filter-out|sort|findstring|foreach|if|or|and|call|eval|value|origin|flavor|shell|strip|patsubst|subst|realpath|abspath|dir|notdir|suffix|basename|addsuffix|addprefix|join|reverse|unique|error|warning|info)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        MakefileLexer { inner }
    }
}

impl Lexer for MakefileLexer {
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
    fn test_makefile_basic() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("all:\n\techo hello");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_comment() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
