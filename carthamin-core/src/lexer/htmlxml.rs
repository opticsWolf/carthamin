use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// HTML lexer supporting tags, attributes, CDATA, and doctype.
pub struct HtmlLexer {
    inner: RegexLexer,
}

impl HtmlLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("HTML");
        inner.aliases = vec!["html", "xhtml"];
        inner.filenames = vec!["*.html", "*.htm", "*.xhtml"];
        inner.mimetypes = vec!["text/html", "application/xhtml+xml"];

        // Root state
        let mut root_rules = Vec::new();

        // Whitespace outside tags
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t\r\n]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // Text outside tags
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[^<]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // DOCTYPE
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<!DOCTYPE[^>]*>", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Comment
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<!--.*?-->", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // CDATA
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<!\[CDATA\[.*?\]\]>", Token::COMMENT_SPECIAL).unwrap(), action: LexerAction::token(Token::COMMENT_SPECIAL) });

        // Opening tag
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<([a-zA-Z][a-zA-Z0-9]*)", Token::NAME_TAG).unwrap(), action: LexerAction::push("tag") });

        // Closing tag
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"</([a-zA-Z][a-zA-Z0-9]*)", Token::NAME_TAG).unwrap(), action: LexerAction::push("closetag") });

        // Self-closing tag
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<([a-zA-Z][a-zA-Z0-9]*)", Token::NAME_TAG).unwrap(), action: LexerAction::push("tag") });

        inner.states.insert("root".to_string(), root_rules);

        // Tag state (inside opening tag)
        inner.states.insert("tag".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t\r\n]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) },
            LexerRule { pattern: TokenPattern::new(r"/>", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r">", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"([a-zA-Z_][a-zA-Z0-9_:-]*)", Token::NAME_ATTRIBUTE).unwrap(), action: LexerAction::push("attribute") },
        ]);

        // Attribute state
        inner.states.insert("attribute".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) },
            LexerRule { pattern: TokenPattern::new(r"=", Token::OPERATOR).unwrap(), action: LexerAction::push("attrvalue") },
            LexerRule { pattern: TokenPattern::new(r"/>|>", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(2) },
        ]);

        // Attribute value state
        inner.states.insert("attrvalue".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r#"([^"'> ]+)"#, Token::LITERAL).unwrap(), action: LexerAction::pop(2) },
            LexerRule { pattern: TokenPattern::new(r#""([^"]*?)""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(2) },
            LexerRule { pattern: TokenPattern::new(r"'([^']*?)'", Token::STRING_SINGLE).unwrap(), action: LexerAction::pop(2) },
        ]);

        // Closing tag state
        inner.states.insert("closetag".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r">", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
        ]);

        HtmlLexer { inner }
    }
}

impl Lexer for HtmlLexer {
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

/// XML lexer supporting tags, attributes, CDATA, and processing instructions.
pub struct XmlLexer {
    inner: RegexLexer,
}

impl XmlLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("XML");
        inner.aliases = vec!["xml"];
        inner.filenames = vec!["*.xml", "*.xsl", "*.xsd", "*.wsdl", "*.svg"];
        inner.mimetypes = vec!["text/xml", "application/xml"];

        // Root state
        let mut root_rules = Vec::new();

        // Whitespace
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t\r\n]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // Text
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[^<]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) });

        // XML declaration / processing instruction
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<\?[^>]*\?>", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Comment
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<!--.*?-->", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // CDATA
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<!\[CDATA\[.*?\]\]>", Token::COMMENT_SPECIAL).unwrap(), action: LexerAction::token(Token::COMMENT_SPECIAL) });

        // DOCTYPE
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<!DOCTYPE[^>]*>", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Opening tag
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<([a-zA-Z_:][a-zA-Z0-9_:.:-]*)", Token::NAME_TAG).unwrap(), action: LexerAction::push("tag") });

        // Closing tag
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"</([a-zA-Z_:][a-zA-Z0-9_:.:-]*)", Token::NAME_TAG).unwrap(), action: LexerAction::push("closetag") });

        inner.states.insert("root".to_string(), root_rules);

        // Tag state
        inner.states.insert("tag".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"[ \t\r\n]+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) },
            LexerRule { pattern: TokenPattern::new(r"/>", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r">", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
            LexerRule { pattern: TokenPattern::new(r"([a-zA-Z_:][a-zA-Z0-9_:.:-]*)", Token::NAME_ATTRIBUTE).unwrap(), action: LexerAction::push("attribute") },
        ]);

        // Attribute state
        inner.states.insert("attribute".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::TEXT).unwrap(), action: LexerAction::token(Token::TEXT) },
            LexerRule { pattern: TokenPattern::new(r"=", Token::OPERATOR).unwrap(), action: LexerAction::push("attrvalue") },
            LexerRule { pattern: TokenPattern::new(r"/>|>", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(2) },
        ]);

        // Attribute value state
        inner.states.insert("attrvalue".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r#""([^"]*?)""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::pop(2) },
            LexerRule { pattern: TokenPattern::new(r"'([^']*?)'", Token::STRING_SINGLE).unwrap(), action: LexerAction::pop(2) },
            LexerRule { pattern: TokenPattern::new(r#"([^"'> ]+)"#, Token::LITERAL).unwrap(), action: LexerAction::pop(2) },
        ]);

        // Closing tag state
        inner.states.insert("closetag".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r">", Token::PUNCTUATION).unwrap(), action: LexerAction::pop(1) },
        ]);

        XmlLexer { inner }
    }
}

impl Lexer for XmlLexer {
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
    fn test_html_basic() {
        let lexer = HtmlLexer::new();
        let tokens = lexer.get_tokens("<div class=\"test\">Hello</div>");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_TAG));
    }

    #[test]
    fn test_html_self_closing() {
        let lexer = HtmlLexer::new();
        let tokens = lexer.get_tokens("<br/>");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_TAG));
    }

    #[test]
    fn test_html_comment() {
        let lexer = HtmlLexer::new();
        let tokens = lexer.get_tokens("<!-- comment -->");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }

    #[test]
    fn test_xml_basic() {
        let lexer = XmlLexer::new();
        let tokens = lexer.get_tokens("<root attr=\"value\"><child/></root>");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME_TAG));
        assert!(token_types.contains(&Token::NAME_ATTRIBUTE));
    }

    #[test]
    fn test_xml_cdata() {
        let lexer = XmlLexer::new();
        let tokens = lexer.get_tokens("<![CDATA[<data>]]></data>");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::COMMENT_SPECIAL));
    }
}
