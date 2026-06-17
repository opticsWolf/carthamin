use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Terraform lexer for HCL/Terraform configuration files.
pub struct TerraformLexer {
    inner: RegexLexer,
}

impl TerraformLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Terraform");
        inner.aliases = vec!["terraform", "hcl", "tf"];
        inner.filenames = vec!["*.tf", "*.tfvars"];
        inner.mimetypes = vec!["text/x-terraform", "text/x-hcl"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Variables
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\{[^}]+\}", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"=|!=|==|<|>|<=|>=|&&|\|\||\+|-|\*|/|%|~|\^|\|", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}\[\]();,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "resource", "data", "output", "variable", "provider", "module",
            "terraform", "locals", "provisioner", "connection", "depends_on",
            "import", "moved", "dynamic", "for_each", "count", "type",
            "required", "optional", "default", "sensitive", "description",
            "validation", "condition", "error_message",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        TerraformLexer { inner }
    }
}

impl Lexer for TerraformLexer {
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
    fn test_terraform_keywords() {
        let lexer = TerraformLexer::new();
        let tokens = lexer.get_tokens("resource \"aws_instance\" \"example\" { }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_terraform_comment() {
        let lexer = TerraformLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
