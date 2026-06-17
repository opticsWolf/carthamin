use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// SQL lexer supporting standard SQL features.
pub struct SqlLexer {
    inner: RegexLexer,
}

impl SqlLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("SQL");
        inner.aliases = vec!["sql"];
        inner.filenames = vec!["*.sql"];
        inner.mimetypes = vec!["text/x-sql"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"--[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'[^']*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""[^"]*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9]+\.[0-9]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9]+", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[();,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"=|!=|<>|<=|>=|<|>", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(SELECT|FROM|WHERE|INSERT|INTO|VALUES|UPDATE|SET|DELETE|CREATE|DROP|ALTER|TABLE|INDEX|VIEW|DATABASE|SCHEMA|JOIN|INNER|LEFT|RIGHT|OUTER|ON|AND|OR|NOT|IN|LIKE|BETWEEN|IS|NULL|AS|ORDER|BY|GROUP|HAVING|LIMIT|OFFSET|UNION|ALL|DISTINCT|EXISTS|CASE|WHEN|THEN|ELSE|END|WITH|RECURSIVE|GRANT|REVOKE|COMMIT|ROLLBACK|BEGIN|TRANSACTION|IF|ELSE|WHILE|LOOP|FOR|CAST|CONVERT|COALESCE|NULLIF|TRUE|FALSE|DEFAULT|CONSTRAINT|PRIMARY|KEY|FOREIGN|REFERENCES|CHECK|UNIQUE|CASCADE|TRIGGER|PROCEDURE|FUNCTION|RETURN|DECLARE|CURSOR|EXEC|EXECUTE)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Built-in functions
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(COUNT|SUM|AVG|MIN|MAX|ABS|CEIL|FLOOR|ROUND|TRUNC|LENGTH|UPPER|LOWER|SUBSTRING|CONCAT|TRIM|LTRIM|RTRIM|REPLACE|COALESCE|NOW|CURRENT_TIMESTAMP|DATE|TIME|YEAR|MONTH|DAY|EXTRACT|FORMAT|PARSE)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        SqlLexer { inner }
    }
}

impl Lexer for SqlLexer {
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
    fn test_sql_basic() {
        let lexer = SqlLexer::new();
        let tokens = lexer.get_tokens("SELECT * FROM users WHERE id = 1;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_sql_comment() {
        let lexer = SqlLexer::new();
        let tokens = lexer.get_tokens("-- this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_sql_string() {
        let lexer = SqlLexer::new();
        let tokens = lexer.get_tokens("SELECT 'hello' FROM dual;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_sql_join() {
        let lexer = SqlLexer::new();
        let tokens = lexer.get_tokens("SELECT a.id, b.name FROM a INNER JOIN b ON a.id = b.a_id;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }
}
