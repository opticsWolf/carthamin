use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// PowerShell lexer for PowerShell scripts.
pub struct PowerShellLexer {
    inner: RegexLexer,
}

impl PowerShellLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("PowerShell");
        inner.aliases = vec!["powershell", "ps", "ps1", "ps2", "psd1", "psd2", "psm1", "psm2"];
        inner.filenames = vec!["*.ps1", "*.ps2", "*.psd1", "*.psd2", "*.psm1", "*.psm2", "*.ps1", "*.ps2", "*.psd1", "*.psd2", "*.psm1", "*.psm2"];
        inner.mimetypes = vec!["text/x-powershell"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_SINGLE).unwrap(), action: LexerAction::token(Token::STRING_SINGLE) });

        // Variables
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[0-9]+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-and|-or|-not|-eq|-ne|-gt|-ge|-lt|-le|-like|-notlike|-match|-notmatch|-contains|-notcontains|-in|-notin|-replace|-split", Token::OPERATOR_WORD).unwrap(), action: LexerAction::token(Token::OPERATOR_WORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%&|^~]=?", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};,.]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "if", "else", "elseif", "foreach", "for", "while", "do", "switch",
            "begin", "process", "end", "try", "catch", "finally", "throw",
            "return", "break", "continue", "exit", "trap", "filter", "parallel",
            "sequence", "using", "in", "as", "is", "class", "enum", "function",
            "param", "dynamicparam", "begin", "process", "end",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Cmdlet aliases
        let cmdlets = [
            "Get-Content", "Set-Content", "Add-Content", "Clear-Content",
            "Get-Item", "Set-Item", "New-Item", "Remove-Item",
            "Get-ChildItem", "Copy-Item", "Move-Item", "Rename-Item",
            "Get-Service", "Start-Service", "Stop-Service", "Restart-Service",
            "Get-Process", "Start-Process", "Stop-Process",
            "Write-Host", "Write-Output", "Write-Error", "Write-Warning",
            "Select-Object", "Where-Object", "ForEach-Object", "Sort-Object",
            "Get-Command", "Get-History", "Invoke-Command",
        ];
        let cmdlet_pattern = format!(r"\b({})\b", cmdlets.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&cmdlet_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Types
        let types = [
            "string", "int", "long", "float", "double", "decimal", "bool",
            "char", "byte", "object", "array", "hashtable", "psobject",
        ];
        let type_pattern = format!(r"\b({})\b", types.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        PowerShellLexer { inner }
    }
}

impl Lexer for PowerShellLexer {
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
    fn test_powershell_keywords() {
        let lexer = PowerShellLexer::new();
        let tokens = lexer.get_tokens("if ($x -eq 1) { Write-Host 'hello' }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_powershell_comment() {
        let lexer = PowerShellLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }
}
