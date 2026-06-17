use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// PostgreSQL lexer for PostgreSQL SQL.
pub struct PostgresLexer {
    inner: RegexLexer,
}

impl PostgresLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("PostgreSQL");
        inner.aliases = vec!["postgresql", "postgres"];
        inner.filenames = vec!["*.pgsql", "*.psql"];
        inner.mimetypes = vec!["text/x-pgsql"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments — FIXED: PostgreSQL uses -- not // for single-line comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"--[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Strings — FIXED: mismatched quotes (ended with " instead of ')
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\x27([^\x27\\]|\\.)*\x27", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // PostgreSQL operators — FIXED: escape | in #| and * in ~*/!~*
        let pg_operators = [
            "&&", "<<", ">>", "~", "~\\*", "!~", "!~\\*", "\\|", "\\&",
            "\\^", "#-", "#\\|", "@@", "@>", "<@", "\\|\\|", "-|-",
        ];
        let operator_pattern = format!(r"({})", pg_operators.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&operator_pattern, Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[(),;:\[\]]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // PostgreSQL keywords
        let pg_keywords = [
            "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE",
            "DELETE", "CREATE", "TABLE", "DROP", "ALTER", "INDEX", "VIEW",
            "DATABASE", "SCHEMA", "USER", "ROLE", "GRANT", "REVOKE",
            "BEGIN", "COMMIT", "ROLLBACK", "TRANSACTION", "ISOLATION",
            "LEVEL", "READ", "WRITE", "COMMITTED", "SERIALIZABLE",
            "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT",
            "UNIQUE", "CHECK", "DEFAULT", "NULL", "NOT", "AND", "OR",
            "LIKE", "ILIKE", "IN", "OUT", "BETWEEN", "EXISTS", "CASE",
            "WHEN", "THEN", "ELSE", "END", "AS", "ORDER", "BY", "GROUP",
            "HAVING", "LIMIT", "OFFSET", "UNION", "INTERSECT", "EXCEPT",
            "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS",
            "NATURAL", "ON", "USING", "DISTINCT", "ALL", "ANY", "SOME",
            "NULLS", "FIRST", "LAST", "WITH", "RECURSIVE", "WINDOW",
            "OVER", "PARTITION", "RANGE", "ROWS", "PRECEDING", "FOLLOWING",
            "CURRENT", "ROW", "UNBOUNDED",
        ];
        let keyword_pattern = format!(r"\b({})\b", pg_keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // PostgreSQL data types
        let pg_types = [
            "INTEGER", "BIGINT", "SMALLINT", "SERIAL", "BIGSERIAL",
            "VARCHAR", "CHAR", "TEXT", "BYTEA", "JSON", "JSONB",
            "XML", "UUID", "CIDR", "INET", "MACADDR", "BIT", "VARBIT",
            "TIMESTAMP", "TIMESTAMPTZ", "DATE", "TIME", "TIMETZ",
            "INTERVAL", "BOOLEAN", "BOOL", "NUMERIC", "DECIMAL",
            "REAL", "DOUBLE", "PRECISION", "MONEY",
            "ARRAY", "OID", "REGCLASS", "REGCOLLATION",
            "REGCONFIG", "REGDICTIONARY", "REGNAMESPACE", "REGOPER",
            "REGOPERATOR", "REGPROCEDURE", "REGROLE", "REGTYPE",
        ];
        let type_pattern = format!(r"\b({})\b", pg_types.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        PostgresLexer { inner }
    }
}

impl Lexer for PostgresLexer {
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
    fn test_postgres_keywords() {
        let lexer = PostgresLexer::new();
        let tokens = lexer.get_tokens("SELECT * FROM users WHERE id = 1");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_postgres_comment() {
        let lexer = PostgresLexer::new();
        let tokens = lexer.get_tokens("-- comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
