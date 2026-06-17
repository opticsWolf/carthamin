use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Django/Jinja lexer for Django and Jinja template files.
pub struct DjangoLexer {
    inner: RegexLexer,
}

impl DjangoLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Django/Jinja");
        inner.aliases = vec!["django", "jinja"];
        inner.filenames = vec!["*.html", "*.jinja", "*.jinja2", "*.djhtml"];
        inner.mimetypes = vec!["application/x-django-templating", "application/x-jinja"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Plain text (everything except {{ and {% )
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[^{{%]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // Variable output {{ ... }}
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{\{", Token::COMMENT).unwrap(), action: LexerAction::push("var") });

        // Comment blocks {# ... #}
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{#.*?#\}", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Django comment blocks {% comment %} ... {% endcomment %}
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{%\s*comment\s*%\}", Token::COMMENT).unwrap(), action: LexerAction::push("comment") });

        // Raw blocks {% raw %} ... {% endraw %}
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{%\s*raw\s*%\}", Token::COMMENT).unwrap(), action: LexerAction::push("raw") });

        // Filter blocks {% filter name %}
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{%\s*filter\s+([a-zA-Z_]\w*)", Token::COMMENT).unwrap(), action: LexerAction::token(Token::NAME) });

        // Block tags {% tag %}
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{%\s*([a-zA-Z_]\w*)", Token::COMMENT).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Left brace (not part of {{ or {% )
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\{", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        inner.states.insert("root".to_string(), root_rules);

        // Variable state (inside {{ ... }}) - use hex escapes for quotes
        inner.states.insert("var".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"(-?)(\}\})", Token::COMMENT).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"(_|true|false|none|True|False|None)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) },
            LexerRule { pattern: TokenPattern::new(r"(in|as|reversed|recursive|not|and|or|is|if|else|import|with|ignore\s+missing)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) },
            LexerRule { pattern: TokenPattern::new(r"(loop|block|super|forloop)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][\w-]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
            LexerRule { pattern: TokenPattern::new(r"\.\w+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
            LexerRule { pattern: TokenPattern::new(r":?\x22([^\x22\\\\]|\\\\.)*\x22", Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) },
            LexerRule { pattern: TokenPattern::new(r":?\x27([^\x27\\\\]|\\\\.)*\x27", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"([{}()\[\]+\-*/%,:~]|[><=]=?|!=)", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) },
            LexerRule { pattern: TokenPattern::new(r"[0-9](\.[0-9]*)?(eE[+-][0-9])?[flFLdD]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) },
        ]);

        // Block state (inside {% ... %})
        inner.states.insert("block".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"(-?)(%\})", Token::COMMENT).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][\w-]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) },
            LexerRule { pattern: TokenPattern::new(r".", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) },
        ]);

        // Comment state (inside {% comment %} ... {% endcomment %})
        inner.states.insert("comment".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"\{%\s*endcomment\s*%\}", Token::COMMENT).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r".", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) },
        ]);

        // Raw state (inside {% raw %} ... {% endraw %})
        inner.states.insert("raw".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"\{%\s*endraw\s*%\}", Token::COMMENT).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r".", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) },
        ]);

        DjangoLexer { inner }
    }
}

impl Lexer for DjangoLexer {
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
    fn test_django_variables() {
        let lexer = DjangoLexer::new();
        let tokens = lexer.get_tokens("{{ variable }}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_django_blocks() {
        let lexer = DjangoLexer::new();
        let tokens = lexer.get_tokens("{% if condition %}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_django_comments() {
        let lexer = DjangoLexer::new();
        let tokens = lexer.get_tokens("{# comment #}");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }
}
