use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Markdown lexer for Markdown text.
pub struct MarkdownLexer {
    inner: RegexLexer,
}

impl MarkdownLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Markdown");
        inner.aliases = vec!["markdown", "md", "mkd", "mdwn", "mdown", "mkdn", "rmd"];
        inner.filenames = vec!["*.md", "*.mkd", "*.mdwn", "*.mdown", "*.mkdn", "*.rmd"];
        inner.mimetypes = vec!["text/x-markdown"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // HTML tags (treated as raw)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<[/]?[a-zA-Z][^>]*>", Token::NAME_TAG).unwrap(), action: LexerAction::token(Token::NAME_TAG) });

        // Headings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#{1,6}\s.*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Horizontal rules
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^[-*_]{3,}\s*$", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Links
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\[([^\]]*)\]\(([^\)]*)\)", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Images
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"!\[([^\]]*)\]\(([^\)]*)\)", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Bold and italic
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\*\*[^*]+\*\*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\*[^*]+\*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"__[^_]+__", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"_[^_]+_", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Code blocks
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^`{3,}.*", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"`[^`]+`", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Blockquotes
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^>\s.*", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Lists
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^\s*[-*+]\s", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^\s*\d+\.\s", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // URLs
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"https?://[^\s]+", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Plain text (default)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[^\n]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        inner.states.insert("root".to_string(), root_rules);

        MarkdownLexer { inner }
    }
}

impl Lexer for MarkdownLexer {
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
    fn test_markdown_headings() {
        let lexer = MarkdownLexer::new();
        let tokens = lexer.get_tokens("# Heading 1\n## Heading 2");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_markdown_code() {
        let lexer = MarkdownLexer::new();
        let tokens = lexer.get_tokens("`code`");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }
}
