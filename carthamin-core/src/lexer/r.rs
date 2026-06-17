use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// R lexer for R source code.
pub struct RLexer {
    inner: RegexLexer,
}

impl RLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("R");
        inner.aliases = vec!["r", "s", "splus"];
        inner.filenames = vec!["*.R", "*.r", ".Rhistory", ".Rprofile", ".Renviron"];
        inner.mimetypes = vec!["text/x-r-source", "text/x-s", "text/x-R", "text/x-r-history", "text/x-r-profile"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?[Li]?|\d+[eE][+-]?\d+[Li]?|\d+[Li]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"<-|->|<<-|->>|==|!=|<=|>=|&&|\|\||~|->>", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/^%*%&|<>]=?", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "function", "if", "else", "for", "while", "repeat", "break",
            "next", "return", "in", "TRUE", "FALSE", "NA", "NULL",
            "Inf", "NaN", "T", "F", "Yes", "No",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtins
        let builtins = [
            "c", "list", "data.frame", "matrix", "array", "factor",
            "as", "is", "class", "names", "names<-", "dim", "dim<-",
            "length", "rep", "seq", "paste", "cat", "print", "summary",
            "str", "head", "tail", "subset", "transform", "within",
            "attach", "detach", "library", "require", "source", "setwd",
            "getwd", "list.files", "dir", "read.csv", "read.table",
            "write.csv", "write.table", "plot", "hist", "boxplot",
            "lm", "glm", "anova", "t.test", "chisq.test", "cor",
            "cov", "var", "sd", "mean", "median", "sum", "min", "max",
            "range", "quantile", "sort", "order", "rank", "unique",
            "intersect", "union", "setdiff", "is.na", "na.omit",
            "complete.cases", "replace", "subset", "transform",
        ];
        let builtin_pattern = format!(r"\b({})\b", builtins.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&builtin_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA_.][a-zA-Z0-9_.]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        RLexer { inner }
    }
}

impl Lexer for RLexer {
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
    fn test_r_keywords() {
        let lexer = RLexer::new();
        let tokens = lexer.get_tokens("if (x > 0) { y <- x * 2 }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_r_comment() {
        let lexer = RLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
