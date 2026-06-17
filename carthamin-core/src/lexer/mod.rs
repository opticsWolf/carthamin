pub mod regex_lexer;
pub mod python;
pub mod javascript;
pub mod htmlxml;
pub mod css;
pub mod rust;
pub mod cpp;
pub mod go;
pub mod java;
pub mod sql;
pub mod bash;
pub mod csharp;
pub mod swift;
pub mod kotlin;
pub mod php;
pub mod ruby;
pub mod lua;
pub mod r;
pub mod json;
pub mod yaml;
pub mod markdown;
pub mod protobuf;
pub mod powershell;
pub mod postgres;
pub mod docker;
pub mod terraform;
pub mod makefile;
pub mod scala;
pub mod julia;
pub mod django;

use crate::token::Token;
use crate::lexer::python::PythonLexer;
use crate::scanner::TokenPattern;

/// A token with its position in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStreamItem {
    pub index: usize,
    pub token_type: Token,
    pub text: String,
}

/// Base trait for all lexers.
pub trait Lexer {
    /// Name of the lexer.
    fn name(&self) -> &str;

    /// Aliases for this lexer.
    fn aliases(&self) -> &[&str];

    /// File name patterns this lexer matches.
    fn filenames(&self) -> &[&str];

    /// MIME types this lexer matches.
    fn mimetypes(&self) -> &[&str];

    /// Analyze text and return a confidence score (0.0 to 1.0).
    fn analyse_text(&self, _text: &str) -> f64 {
        0.0
    }

    /// Tokenize the input text, yielding (index, token_type, text) tuples.
    fn get_tokens_unprocessed(&self, _text: &str) -> Vec<TokenStreamItem> {
        Vec::new()
    }

    /// Tokenize the input text, yielding (token_type, text) tuples.
    fn get_tokens(&self, text: &str) -> Vec<(Token, String)> {
        self.get_tokens_unprocessed(text)
            .into_iter()
            .map(|item| (item.token_type, item.text))
            .collect()
    }
}

/// A rule in a RegexLexer state.
#[derive(Debug, Clone)]
pub struct LexerRule {
    pub pattern: TokenPattern,
    pub action: LexerAction,
}

impl LexerRule {
    /// Create a rule with a simple token action.
    pub fn token(pattern: &str, token: Token) -> Result<Self, regex::Error> {
        Ok(LexerRule {
            pattern: TokenPattern::new(pattern, token)?,
            action: LexerAction::Noop,
        })
    }

    /// Create a rule with a push action.
    pub fn push(pattern: &str, token: Token, state: &str) -> Result<Self, regex::Error> {
        Ok(LexerRule {
            pattern: TokenPattern::new(pattern, token)?,
            action: LexerAction::Push(state.to_string()),
        })
    }

    /// Create a rule with both token and push.
    pub fn token_and_push(pattern: &str, token: Token, state: &str) -> Result<Self, regex::Error> {
        Ok(LexerRule {
            pattern: TokenPattern::new(pattern, token)?,
            action: LexerAction::TokenAndPush(token, state.to_string()),
        })
    }
}

/// Action to take when a rule matches.
#[derive(Debug, Clone)]
pub enum LexerAction {
    /// Assign a single token type.
    Token(Token),
    /// Assign different token types to capture groups.
    ByGroups(Vec<Option<Token>>),
    /// Push a new state onto the stack.
    Push(String),
    /// Pop N states from the stack.
    PopN(usize),
    /// Push a state and assign a token.
    TokenAndPush(Token, String),
    /// No action (skip this match).
    Noop,
    // Delegate to another lexer (not yet implemented).
    // Using(Token),
}

impl LexerAction {
    /// Create a Token action.
    pub fn token(t: Token) -> Self { LexerAction::Token(t) }
    /// Create a Push action.
    pub fn push(state: &str) -> Self { LexerAction::Push(state.to_string()) }
    /// Create a Pop action.
    pub fn pop(n: usize) -> Self { LexerAction::PopN(n) }
}

/// A regex-based lexer with named states and rules.
pub struct RegexLexer {
    pub name: String,
    pub aliases: Vec<&'static str>,
    pub filenames: Vec<&'static str>,
    pub mimetypes: Vec<&'static str>,
    /// Named states, each containing a list of rules.
    pub states: std::collections::HashMap<String, Vec<LexerRule>>,
    /// Default initial state.
    pub initial_state: String,
}

impl RegexLexer {
    pub fn new(name: &str) -> Self {
        RegexLexer {
            name: name.to_string(),
            aliases: Vec::new(),
            filenames: Vec::new(),
            mimetypes: Vec::new(),
            states: std::collections::HashMap::new(),
            initial_state: "root".to_string(),
        }
    }

    pub fn add_state(&mut self, name: &str, rules: Vec<LexerRule>) {
        self.states.insert(name.to_string(), rules);
    }

    pub fn add_rule(&mut self, state: &str, rule: LexerRule) {
        self.states.entry(state.to_string()).or_default().push(rule);
    }

    /// Build a scanner from a state's rules.
    fn build_scanner(&self, state: &str) -> Option<crate::scanner::RegexScanner> {
        let rules = self.states.get(state)?;
        let mut scanner = crate::scanner::RegexScanner::new();
        for rule in rules {
            scanner.add(rule.pattern.clone());
        }
        Some(scanner)
    }

    /// Core tokenization: process text through the state machine.
    pub fn tokenize(&self, text: &str) -> Vec<TokenStreamItem> {
        let mut tokens = Vec::new();
        let mut state_stack = vec![self.initial_state.clone()];
        let mut pos = 0;
        let mut index = 0;

        while pos < text.len() {
            let current_state = state_stack.last().unwrap();
            let rules = match self.states.get(current_state) {
                Some(r) => r,
                None => break, // Unknown state
            };

            let mut matched = false;

            for rule in rules {
                if let Some(m) = rule.pattern.regex.find(&text[pos..]) {
                    // Check if this match starts at current position or is the earliest
                    if m.start() == 0 {
                        let end = pos + m.end();
                        let matched_text = &text[pos..end];

                        // Apply action
                        match &rule.action {
                            LexerAction::Token(token) => {
                                tokens.push(TokenStreamItem {
                                    index,
                                    token_type: *token,
                                    text: matched_text.to_string(),
                                });
                            }
                            LexerAction::ByGroups(groups) => {
                                let captures = rule.pattern.regex.captures(&text[pos..]).unwrap();
                                for (i, _group_token) in groups.iter().enumerate() {
                                    if let Some(Some(token)) = groups.get(i) {
                                        if let Some(c) = captures.get(i + 1) {
                                            let group_text = c.as_str();
                                            if !group_text.is_empty() {
                                                tokens.push(TokenStreamItem {
                                                    index,
                                                    token_type: *token,
                                                    text: group_text.to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                                // If no groups produced tokens, use the whole match
                                if tokens.is_empty() || tokens.last().map(|t| t.index).unwrap_or(0) == index {
                                    tokens.push(TokenStreamItem {
                                        index,
                                        token_type: Token::TEXT,
                                        text: matched_text.to_string(),
                                    });
                                }
                            }
                            _ => {
                                tokens.push(TokenStreamItem {
                                    index,
                                    token_type: rule.pattern.token,
                                    text: matched_text.to_string(),
                                });
                            }
                        }

                        // Handle state changes
                        if let LexerAction::Push(state) = &rule.action {
                            state_stack.push(state.clone());
                        } else if let LexerAction::PopN(n) = &rule.action {
                            for _ in 0..*n {
                                state_stack.pop();
                            }
                        } else if let LexerAction::TokenAndPush(_, state) = &rule.action {
                            state_stack.push(state.clone());
                        }

                        index += 1;
                        pos = if end > pos { end } else { pos + 1 };
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                // No rule matched at current position — consume one character as Text
                let ch_end = text[pos..].char_indices().nth(1)
                    .map(|(i, _)| pos + i)
                    .unwrap_or(pos + 1);
                tokens.push(TokenStreamItem {
                    index,
                    token_type: Token::TEXT,
                    text: text[pos..ch_end].to_string(),
                });
                index += 1;
                pos = ch_end;
            }
        }

        tokens
    }
}

impl Lexer for RegexLexer {
    fn name(&self) -> &str {
        &self.name
    }

    fn aliases(&self) -> &[&str] {
        &self.aliases
    }

    fn filenames(&self) -> &[&str] {
        &self.filenames
    }

    fn mimetypes(&self) -> &[&str] {
        &self.mimetypes
    }

    fn get_tokens_unprocessed(&self, text: &str) -> Vec<TokenStreamItem> {
        self.tokenize(text)
    }
}

/// Generate an optimized keyword regex using words().
pub fn words(keywords: &[&str], token: Token, prefix: &str, suffix: &str) -> LexerRule {
    let pattern_str = crate::regexopt::regex_opt(keywords, prefix, suffix);
    let pattern = TokenPattern::new(&pattern_str, token).unwrap();
    LexerRule {
        pattern,
        action: LexerAction::Token(token),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_lexer_basic() {
        let mut lexer = RegexLexer::new("test");
        lexer.aliases.push("test");

        // Add root state with keyword and identifier rules
        lexer.add_rule("root", LexerRule {
            pattern: TokenPattern::new(r"(if|else|while)", Token::KEYWORD).unwrap(),
            action: LexerAction::token(Token::KEYWORD),
        });
        lexer.add_rule("root", LexerRule {
            pattern: TokenPattern::new(r"[a-zA-Z_]\w*", Token::NAME).unwrap(),
            action: LexerAction::token(Token::NAME),
        });
        lexer.add_rule("root", LexerRule {
            pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(),
            action: LexerAction::token(Token::WHITESPACE),
        });

        let tokens = lexer.tokenize("if x == 5");
        let token_types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NAME));
        assert!(token_types.contains(&Token::WHITESPACE));
    }
}
