//! ExtendedRegexLexer and related constructs.
//!
//! This module provides:
//! - `ExtendedRegexLexer` — context-aware lexer with mutable state tracking
//! - `DelegatingLexer` — two-lexer pipeline (root + language)
//! - `LexerContext` — mutable lexer position/state for callbacks
//! - `bygroups()`, `using()`, `include()`, `combined()`, `inherit` helpers
//! - `LexerFactory` trait for dynamic lexer instantiation

use std::collections::HashMap;
use std::sync::Arc;

use crate::token::Token;
use crate::lexer::{Lexer, LexerRule, LexerAction, TokenStreamItem};
use crate::scanner::TokenPattern;

// ---------------------------------------------------------------------------
// LexerContext — mutable position/state for callbacks
// ---------------------------------------------------------------------------

/// Holds mutable lexer state during tokenization.
/// Used by `ExtendedRegexLexer` and callbacks (`bygroups`, `using`).
pub struct LexerContext {
    /// Full source text
    pub text: String,
    /// Current byte position
    pub pos: usize,
    /// End boundary (exclusive); 0 means no limit
    pub end: usize,
    /// Stack of active state names
    pub stack: Vec<String>,
}

impl LexerContext {
    pub fn new(text: &str) -> Self {
        LexerContext {
            text: text.to_string(),
            pos: 0,
            end: text.len(),
            stack: vec!["root".to_string()],
        }
    }

    pub fn with_stack(text: &str, stack: Vec<String>) -> Self {
        LexerContext {
            text: text.to_string(),
            pos: 0,
            end: text.len(),
            stack,
        }
    }
}

// ---------------------------------------------------------------------------
// LexerFactory — dynamic lexer creation
// ---------------------------------------------------------------------------

/// Trait for creating lexer instances by type name.
pub trait LexerFactory: Send + Sync {
    fn create(&self, type_name: &str) -> Option<Box<dyn Lexer>>;
}

/// Registry-based factory that maps type names to constructors.
pub struct RegistryFactory {
    constructors: HashMap<String, Box<dyn Fn() -> Box<dyn Lexer> + Send + Sync>>,
}

impl RegistryFactory {
    pub fn new() -> Self {
        RegistryFactory {
            constructors: HashMap::new(),
        }
    }

    pub fn register<T: Lexer + 'static, F: Fn() -> Box<T> + Send + Sync + 'static>(&mut self, name: &str, f: F) {
        self.constructors.insert(name.to_string(), Box::new(move || f() as Box<dyn Lexer>));
    }
}

impl LexerFactory for RegistryFactory {
    fn create(&self, type_name: &str) -> Option<Box<dyn Lexer>> {
        self.constructors.get(type_name).map(|f| f())
    }
}

// ---------------------------------------------------------------------------
// ExtendedRegexLexer
// ---------------------------------------------------------------------------

/// A RegexLexer that uses a context object to store its state.
///
/// Unlike `RegexLexer`, this variant supports callbacks that can
/// inspect and modify the lexer's position and state stack during
/// tokenization — required for `bygroups()` and `using()`.
pub struct ExtendedRegexLexer {
    pub name: String,
    pub aliases: Vec<&'static str>,
    pub filenames: Vec<&'static str>,
    pub mimetypes: Vec<&'static str>,
    /// Named states, each containing a list of rules.
    pub states: HashMap<String, Vec<ExtendedRule>>,
    /// Default initial state.
    pub initial_state: String,
    /// Lexer factory for `using()` callbacks
    pub factory: Option<Arc<dyn LexerFactory>>,
}

/// An extended rule with optional state transition and callback.
#[derive(Debug, Clone)]
pub struct ExtendedRule {
    pub pattern: TokenPattern,
    pub action: ExtendedAction,
    /// State transition: push state name, pop N, or none
    pub new_state: Option<ExtendedState>,
}

/// Extended state transition.
#[derive(Debug, Clone)]
pub enum ExtendedState {
    /// Push a named state
    Push(String),
    /// Push multiple states
    PushMulti(Vec<String>),
    /// Pop N states (negative value from Pygments)
    Pop(usize),
    /// Pop all but root
    PopAll,
    /// Push current state again
    PushCurrent,
}

/// Extended action: simple token, bygroups callback, or using callback.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedAction {
    /// Assign a single token type
    Token(Token),
    /// bygroups — emit tokens for each capture group
    ByGroups(Vec<Option<Token>>),
    /// using — delegate to another lexer
    Using {
        lexer_name: String,
        initial_states: Option<Vec<String>>,
    },
    /// using(this) — re-tokenize with the current lexer
    UsingSelf {
        initial_states: Option<Vec<String>>,
    },
    /// No action (skip)
    Noop,
}

impl ExtendedRegexLexer {
    pub fn new(name: &str) -> Self {
        ExtendedRegexLexer {
            name: name.to_string(),
            aliases: Vec::new(),
            filenames: Vec::new(),
            mimetypes: Vec::new(),
            states: HashMap::new(),
            initial_state: "root".to_string(),
            factory: None,
        }
    }

    pub fn with_factory(name: &str, factory: Arc<dyn LexerFactory>) -> Self {
        ExtendedRegexLexer {
            factory: Some(factory),
            ..Self::new(name)
        }
    }

    pub fn add_state(&mut self, name: &str, rules: Vec<ExtendedRule>) {
        self.states.insert(name.to_string(), rules);
    }

    pub fn add_rule(&mut self, state: &str, rule: ExtendedRule) {
        self.states.entry(state.to_string()).or_default().push(rule);
    }

    /// Resolve `include('state')` directives by expanding them inline.
    /// This is called after all states are defined.
    pub fn resolve_includes(&mut self) {
        let mut processed: HashMap<String, Vec<ExtendedRule>> = HashMap::new();

        for (state_name, rules) in &self.states {
            Self::resolve_state(self, state_name, &mut processed);
        }

        self.states = processed;
    }

    fn resolve_state(
        lexer: &ExtendedRegexLexer,
        state_name: &str,
        processed: &mut HashMap<String, Vec<ExtendedRule>>,
    ) {
        if processed.contains_key(state_name) {
            return;
        }

        let rules = match lexer.states.get(state_name) {
            Some(r) => r,
            None => return,
        };

        let mut expanded = Vec::new();
        for rule in rules {
            // Check if this rule is an "include" directive
            // In our Rust model, include is represented as a special TokenPattern
            // with an empty regex and a specific marker token
            if rule.action == ExtendedAction::Noop
                && rule.pattern.token == Token::OTHER
                && rule.new_state.is_none()
            {
                // The pattern text itself holds the include target
                // We use the regex pattern string as a convention
                let include_target = rule.pattern.regex.as_str();
                if !include_target.is_empty() {
                    Self::resolve_state(lexer, include_target, processed);
                    if let Some(included_rules) = processed.get(include_target) {
                        expanded.extend(included_rules.iter().cloned());
                    }
                }
            } else {
                expanded.push(rule.clone());
            }
        }

        processed.insert(state_name.to_string(), expanded);
    }

    /// Resolve `inherit` directives by merging parent states.
    /// Given a parent lexer's states, merge them into this lexer's states.
    pub fn resolve_inherit(&mut self, parent: &ExtendedRegexLexer) {
        for (state_name, parent_rules) in &parent.states {
            if let Some(child_rules) = self.states.get_mut(state_name) {
                // Check if child has an "inherit" marker
                let inherit_idx = child_rules.iter().position(|r| {
                    r.action == ExtendedAction::Noop
                        && r.pattern.token == Token::ESCAPE
                        && r.new_state.is_none()
                        && r.pattern.regex.as_str() == "__inherit__"
                });

                if let Some(idx) = inherit_idx {
                    // Insert parent rules at the inherit marker position
                    child_rules.splice(idx..idx + 1, parent_rules.iter().cloned());
                }
            } else {
                // State not defined in child — inherit entirely
                self.states
                    .insert(state_name.to_string(), parent_rules.clone());
            }
        }
    }

    /// Resolve `combined('state1', 'state2', ...)` by creating a merged state.
    pub fn add_combined_state(&mut self, name: &str, source_states: &[&str]) {
        let mut combined_rules = Vec::new();
        for src in source_states {
            if let Some(rules) = self.states.get(*src) {
                combined_rules.extend(rules.iter().cloned());
            }
        }
        self.states.insert(name.to_string(), combined_rules);
    }

    /// Core tokenization with context-aware state management.
    pub fn tokenize(&self, text: &str) -> Vec<TokenStreamItem> {
        let mut ctx = LexerContext::new(text);
        let mut tokens = Vec::new();
        let tokendefs = &self.states;

        let mut statetokens: Option<&[ExtendedRule]> = tokendefs.get("root").map(|v| v.as_slice());
        if statetokens.is_none() {
            return tokens;
        }

        loop {
            let current_rules = match statetokens {
                Some(r) => r,
                None => break,
            };

            let mut matched = false;

            for rule in current_rules {
                let rest = &text[ctx.pos..ctx.end.min(text.len())];
                if let Some(m) = rule.pattern.regex.find(rest) {
                    if m.start() == 0 {
                        let match_end = ctx.pos + m.end();
                        let matched_text = &text[ctx.pos..match_end];

                        // Apply action
                        match &rule.action {
                            ExtendedAction::Token(token) => {
                                tokens.push(TokenStreamItem {
                                    index: tokens.len(),
                                    token_type: *token,
                                    text: matched_text.to_string(),
                                });
                                ctx.pos = match_end;
                            }
                            ExtendedAction::ByGroups(groups) => {
                                let captures = rule.pattern.regex.captures(rest).unwrap();
                                let mut any_group = false;
                                for (i, group_token) in groups.iter().enumerate() {
                                    if let Some(Some(token)) = groups.get(i) {
                                        if let Some(c) = captures.get(i + 1) {
                                            let group_text = c.as_str();
                                            if !group_text.is_empty() {
                                                tokens.push(TokenStreamItem {
                                                    index: tokens.len(),
                                                    token_type: *token,
                                                    text: group_text.to_string(),
                                                });
                                                any_group = true;
                                            }
                                        }
                                    }
                                }
                                if !any_group {
                                    tokens.push(TokenStreamItem {
                                        index: tokens.len(),
                                        token_type: Token::TEXT,
                                        text: matched_text.to_string(),
                                    });
                                }
                                ctx.pos = match_end;
                            }
                            ExtendedAction::Using { lexer_name, initial_states } => {
                                // Delegate to another lexer
                                if let Some(factory) = &self.factory {
                                    if let Some(sub_lexer) = factory.create(lexer_name) {
                                        let sub_tokens = sub_lexer.get_tokens_unprocessed(matched_text);
                                        for sub_item in sub_tokens {
                                            tokens.push(TokenStreamItem {
                                                index: tokens.len(),
                                                token_type: sub_item.token_type,
                                                text: sub_item.text,
                                            });
                                        }
                                    }
                                }
                                ctx.pos = match_end;
                            }
                            ExtendedAction::UsingSelf { initial_states: _ } => {
                                // Re-tokenize with current lexer
                                let sub_tokens = self.tokenize(matched_text);
                                for sub_item in sub_tokens {
                                    tokens.push(TokenStreamItem {
                                        index: tokens.len(),
                                        token_type: sub_item.token_type,
                                        text: sub_item.text,
                                    });
                                }
                                ctx.pos = match_end;
                            }
                            ExtendedAction::Noop => {
                                // Skip
                                ctx.pos = match_end;
                            }
                        }

                        // Handle state transitions
                        if let Some(new_state) = &rule.new_state {
                            match new_state {
                                ExtendedState::Push(state) => {
                                    ctx.stack.push(state.clone());
                                }
                                ExtendedState::PushMulti(states) => {
                                    for s in states {
                                        ctx.stack.push(s.clone());
                                    }
                                }
                                ExtendedState::Pop(n) => {
                                    let pop_count = (*n).min(ctx.stack.len() - 1);
                                    for _ in 0..pop_count {
                                        ctx.stack.pop();
                                    }
                                }
                                ExtendedState::PopAll => {
                                    ctx.stack = vec!["root".to_string()];
                                }
                                ExtendedState::PushCurrent => {
                                    if let Some(last) = ctx.stack.last() {
                                        ctx.stack.push(last.clone());
                                    }
                                }
                            }
                            statetokens = tokendefs.get(ctx.stack.last().unwrap()).map(|v| v.as_slice());
                        }

                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                // No rule matched at current position
                if ctx.pos >= ctx.end.min(text.len()) {
                    break;
                }

                let ch = text[ctx.pos..].chars().next();
                if ch == Some('\n') {
                    // At EOL, reset state to root
                    ctx.stack = vec!["root".to_string()];
                    statetokens = tokendefs.get("root").map(|v| v.as_slice());
                    tokens.push(TokenStreamItem {
                        index: tokens.len(),
                        token_type: Token::WHITESPACE,
                        text: "\n".to_string(),
                    });
                    ctx.pos += 1;
                    continue;
                }

                // Error token for unmatched character
                let ch_end = text[ctx.pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| ctx.pos + i)
                    .unwrap_or(text.len());
                tokens.push(TokenStreamItem {
                    index: tokens.len(),
                    token_type: Token::ERROR,
                    text: text[ctx.pos..ch_end].to_string(),
                });
                ctx.pos = ch_end;
            }
        }

        tokens
    }
}

impl Lexer for ExtendedRegexLexer {
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

// ---------------------------------------------------------------------------
// DelegatingLexer
// ---------------------------------------------------------------------------

/// A lexer that delegates tokenization between two lexers.
///
/// First scans with `_language_lexer`, then re-scans all `Other` tokens
/// with `_root_lexer`. Used for template lexers (e.g., HTML + embedded JS).
pub struct DelegatingLexer {
    pub name: String,
    pub aliases: Vec<&'static str>,
    pub filenames: Vec<&'static str>,
    pub mimetypes: Vec<&'static str>,
    root_lexer: Box<dyn Lexer>,
    language_lexer: Box<dyn Lexer>,
    /// Token type to re-scan (default: Token::OTHER)
    needle: Token,
}

impl DelegatingLexer {
    pub fn new(
        name: &str,
        root_lexer: Box<dyn Lexer>,
        language_lexer: Box<dyn Lexer>,
    ) -> Self {
        DelegatingLexer {
            name: name.to_string(),
            aliases: Vec::new(),
            filenames: Vec::new(),
            mimetypes: Vec::new(),
            root_lexer,
            language_lexer,
            needle: Token::OTHER,
        }
    }

    pub fn with_needle(
        name: &str,
        root_lexer: Box<dyn Lexer>,
        language_lexer: Box<dyn Lexer>,
        needle: Token,
    ) -> Self {
        DelegatingLexer { needle, ..Self::new(name, root_lexer, language_lexer) }
    }

    /// Tokenize: first with language lexer, then re-scan contiguous needle tokens with root lexer.
    ///
    /// Matches Pygments' DelegatingLexer: accumulates all needle (OTHER) text into one buffer,
    /// tracks non-needle tokens with their positions, re-scans buffer with root lexer,
    /// then splices non-needle tokens back in via do_insertions.
    pub fn tokenize(&self, text: &str) -> Vec<TokenStreamItem> {
        // Phase 1: scan with language lexer
        let lang_tokens = self.language_lexer.get_tokens_unprocessed(text);

        // Phase 2: collect insertions (non-needle tokens) and a single buffer of needle text
        let mut insertions: Vec<(usize, Vec<TokenStreamItem>)> = Vec::new();
        let mut buffered = String::new();
        let mut lng_buffer: Vec<TokenStreamItem> = Vec::new();

        for item in lang_tokens {
            if item.token_type == self.needle {
                if !lng_buffer.is_empty() {
                    insertions.push((buffered.len(), lng_buffer));
                    lng_buffer = Vec::new();
                }
                buffered.push_str(&item.text);
            } else {
                lng_buffer.push(TokenStreamItem {
                    index: 0,
                    token_type: item.token_type,
                    text: item.text.clone(),
                });
            }
        }
        if !lng_buffer.is_empty() {
            insertions.push((buffered.len(), lng_buffer));
        }

        // Phase 3: re-scan entire buffer with root lexer
        let root_tokens = self.root_lexer.get_tokens_unprocessed(&buffered);

        // Phase 4: do_insertions — splice insertions into root tokens at char positions
        self.do_insertions(root_tokens, insertions)
    }

    /// Splice insertion tokens into the main token stream at character positions.
    fn do_insertions(
        &self,
        tokens: Vec<TokenStreamItem>,
        insertions: Vec<(usize, Vec<TokenStreamItem>)>,
    ) -> Vec<TokenStreamItem> {
        if insertions.is_empty() {
            return tokens;
        }

        let mut result = Vec::new();
        let mut char_pos = 0usize;
        let mut ins_iter = insertions.into_iter().peekable();

        for token in tokens {
            // Insert any pending insertions at or before current position
            while let Some(&(pos, ref ins_tokens)) = ins_iter.peek() {
                if pos <= char_pos {
                    result.extend(ins_tokens.iter().cloned());
                    ins_iter.next();
                } else {
                    break;
                }
            }
            result.push(token.clone());
            char_pos += token.text.len();
        }

        // Flush any remaining insertions
        for (_, ins_tokens) in ins_iter {
            result.extend(ins_tokens);
        }

        result
    }
}

impl Lexer for DelegatingLexer {
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

// ---------------------------------------------------------------------------
// Helper functions for lexer construction
// ---------------------------------------------------------------------------

/// Create an `include('state')` rule that references another state.
/// The include is resolved at construction time via `resolve_includes()`.
pub fn include(state: &str) -> ExtendedRule {
    ExtendedRule {
        pattern: TokenPattern::new(state, Token::OTHER).unwrap_or_else(|_| {
            // Fallback for non-regex state names
            TokenPattern::new(r"__include_placeholder__", Token::OTHER).unwrap()
        }),
        action: ExtendedAction::Noop,
        new_state: None,
    }
}

/// Create an `inherit` marker that pulls in parent state rules.
/// Resolved via `resolve_inherit()`.
pub fn inherit() -> ExtendedRule {
    ExtendedRule {
        pattern: TokenPattern::new(r"__inherit__", Token::ESCAPE).unwrap(),
        action: ExtendedAction::Noop,
        new_state: None,
    }
}

/// Create a `bygroups(...)` action from token types.
/// Each token corresponds to capture group 1, 2, 3, etc.
/// Use `None` to skip a group.
pub fn bygroups(tokens: Vec<Option<Token>>) -> ExtendedAction {
    ExtendedAction::ByGroups(tokens)
}

/// Create a `using(OtherLexer)` action.
pub fn using(lexer_name: &str) -> ExtendedAction {
    ExtendedAction::Using {
        lexer_name: lexer_name.to_string(),
        initial_states: None,
    }
}

/// Create a `using(OtherLexer, state='state')` action.
pub fn using_with_state(lexer_name: &str, state: &str) -> ExtendedAction {
    ExtendedAction::Using {
        lexer_name: lexer_name.to_string(),
        initial_states: Some(vec!["root".to_string(), state.to_string()]),
    }
}

/// Create a `using(this)` action — re-tokenize with current lexer.
pub fn using_this() -> ExtendedAction {
    ExtendedAction::UsingSelf {
        initial_states: None,
    }
}

/// Create a `combined('state1', 'state2', ...)` state transition.
pub fn combined(states: &[&str]) -> ExtendedState {
    ExtendedState::PushMulti(
        states.iter().map(|s| s.to_string()).collect(),
    )
}

/// Create a rule with bygroups and state transition.
pub fn rule_bygroups_push(
    pattern: &str,
    groups: Vec<Option<Token>>,
    state: &str,
) -> Result<ExtendedRule, regex::Error> {
    Ok(ExtendedRule {
        pattern: TokenPattern::new(pattern, Token::TEXT)?,
        action: bygroups(groups),
        new_state: Some(ExtendedState::Push(state.to_string())),
    })
}

/// Create a simple token rule with optional state transition.
pub fn rule_token_push(
    pattern: &str,
    token: Token,
    state: Option<&str>,
) -> Result<ExtendedRule, regex::Error> {
    Ok(ExtendedRule {
        pattern: TokenPattern::new(pattern, token)?,
        action: ExtendedAction::Token(token),
        new_state: state.map(|s| ExtendedState::Push(s.to_string())),
    })
}

/// Create a rule that pops N states.
pub fn rule_pop(pattern: &str, token: Token, n: usize) -> Result<ExtendedRule, regex::Error> {
    Ok(ExtendedRule {
        pattern: TokenPattern::new(pattern, token)?,
        action: ExtendedAction::Token(token),
        new_state: Some(ExtendedState::Pop(n)),
    })
}

/// Convert a `RegexLexer`-style `LexerRule` to `ExtendedRule`.
pub fn from_lexer_rule(rule: &LexerRule) -> ExtendedRule {
    ExtendedRule {
        pattern: rule.pattern.clone(),
        action: match &rule.action {
            LexerAction::Token(t) => ExtendedAction::Token(*t),
            LexerAction::ByGroups(groups) => ExtendedAction::ByGroups(groups.clone()),
            LexerAction::Push(s) => {
                // Token action + push state
                ExtendedAction::Token(rule.pattern.token)
            }
            LexerAction::PopN(n) => ExtendedAction::Noop,
            LexerAction::TokenAndPush(t, s) => ExtendedAction::Token(*t),
            LexerAction::Noop => ExtendedAction::Noop,
            LexerAction::Using(name) => ExtendedAction::Using {
                lexer_name: name.clone(),
                initial_states: None,
            },
            LexerAction::Default(_) => ExtendedAction::Noop,
        },
        new_state: match &rule.action {
            LexerAction::Push(s) => Some(ExtendedState::Push(s.clone())),
            LexerAction::PopN(n) => Some(ExtendedState::Pop(*n)),
            LexerAction::TokenAndPush(_, s) => Some(ExtendedState::Push(s.clone())),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_lexer_basic() {
        let mut lexer = ExtendedRegexLexer::new("test");

        lexer.add_rule("root", rule_token_push(r"(if|else|while)", Token::KEYWORD, None).unwrap());
        lexer.add_rule("root", rule_token_push(r"[a-zA-Z_]\w*", Token::NAME, None).unwrap());
        lexer.add_rule("root", rule_token_push(r"\s+", Token::WHITESPACE, None).unwrap());

        let tokens = lexer.tokenize("if x == 5");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::KEYWORD));
        assert!(types.contains(&Token::NAME));
        assert!(types.contains(&Token::WHITESPACE));
    }

    #[test]
    fn test_bygroups() {
        let mut lexer = ExtendedRegexLexer::new("bygroups_test");

        // Match "function fooName" → Keyword.Declaration + Name.Function
        lexer.add_rule("root", ExtendedRule {
            pattern: TokenPattern::new(r"(function)\s+(\w+)", Token::TEXT).unwrap(),
            action: bygroups(vec![Some(Token::KEYWORD_DECLARATION), Some(Token::NAME_FUNCTION)]),
            new_state: None,
        });
        lexer.add_rule("root", rule_token_push(r"\s+", Token::WHITESPACE, None).unwrap());

        let tokens = lexer.tokenize("function main");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, Token::KEYWORD_DECLARATION);
        assert_eq!(tokens[0].text, "function");
        assert_eq!(tokens[1].token_type, Token::NAME_FUNCTION);
        assert_eq!(tokens[1].text, "main");
    }

    #[test]
    fn test_bygroups_with_skipped_group() {
        let mut lexer = ExtendedRegexLexer::new("bygroups_skip");

        // Match "import foo as bar" → Keyword + (skip) + Name + (skip) + Keyword + Name
        lexer.add_rule("root", ExtendedRule {
            pattern: TokenPattern::new(r"(import)\s+(\w+)(\s+as\s+)(\w+)", Token::TEXT).unwrap(),
            action: bygroups(vec![
                Some(Token::KEYWORD),
                Some(Token::NAME),
                None, // skip whitespace
                Some(Token::NAME),
            ]),
            new_state: None,
        });

        let tokens = lexer.tokenize("import foo as bar");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::KEYWORD));
        assert_eq!(tokens.iter().filter(|t| t.token_type == Token::NAME).count(), 2);
    }

    #[test]
    fn test_state_push_pop() {
        let mut lexer = ExtendedRegexLexer::new("state_test");

        lexer.add_rule("root", ExtendedRule {
            pattern: TokenPattern::new(r"<", Token::PUNCTUATION).unwrap(),
            action: ExtendedAction::Token(Token::PUNCTUATION),
            new_state: Some(ExtendedState::Push("tag".to_string())),
        });
        lexer.add_rule("tag", ExtendedRule {
            pattern: TokenPattern::new(r">", Token::PUNCTUATION).unwrap(),
            action: ExtendedAction::Token(Token::PUNCTUATION),
            new_state: Some(ExtendedState::Pop(1)),
        });
        lexer.add_rule("tag", rule_token_push(r"[a-zA-Z_]\w*", Token::NAME, None).unwrap());
        lexer.add_rule("root", rule_token_push(r"[^\s<]+", Token::TEXT, None).unwrap());

        let tokens = lexer.tokenize("<div> hello");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert_eq!(types[0], Token::PUNCTUATION); // <
        assert_eq!(types[1], Token::NAME);         // div
        assert_eq!(types[2], Token::PUNCTUATION);  // >
    }

    #[test]
    fn test_include_resolution() {
        let mut lexer = ExtendedRegexLexer::new("include_test");

        // Define comments state
        lexer.add_rule("comments", rule_token_push(r"//.*$", Token::COMMENT, None).unwrap());

        // Root includes comments
        lexer.add_rule("root", include("comments"));
        lexer.add_rule("root", rule_token_push(r"\w+", Token::NAME, None).unwrap());

        lexer.resolve_includes();

        let tokens = lexer.tokenize("hello // world");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::NAME));
        assert!(types.contains(&Token::COMMENT));
    }

    #[test]
    fn test_inherit_resolution() {
        let mut parent = ExtendedRegexLexer::new("parent");
        parent.add_rule("root", rule_token_push(r"(if|else)", Token::KEYWORD, None).unwrap());
        parent.add_rule("root", rule_token_push(r"\w+", Token::NAME, None).unwrap());

        let mut child = ExtendedRegexLexer::new("child");
        // Inherit parent's root state, then add child-specific rules
        child.add_rule("root", inherit());
        child.add_rule("root", rule_token_push(r"(match|switch)", Token::KEYWORD, None).unwrap());

        child.resolve_inherit(&parent);

        let tokens = child.tokenize("if match");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_delegating_lexer() {
        // Root lexer (HTML-like) — re-scans Other tokens
        let mut root = crate::lexer::RegexLexer::new("root");
        root.add_rule("root", crate::lexer::LexerRule {
            pattern: TokenPattern::new(r"<[^>]+>", Token::NAME_TAG).unwrap(),
            action: LexerAction::Token(Token::NAME_TAG),
        });
        root.add_rule("root", crate::lexer::LexerRule {
            pattern: TokenPattern::new(r".", Token::TEXT).unwrap(),
            action: LexerAction::Token(Token::TEXT),
        });

        // Language lexer (simple code) — produces Other for non-code parts
        let mut lang = crate::lexer::RegexLexer::new("lang");
        lang.add_rule("root", crate::lexer::LexerRule {
            pattern: TokenPattern::new(r"(if|return)\b", Token::KEYWORD).unwrap(),
            action: LexerAction::Token(Token::KEYWORD),
        });
        lang.add_rule("root", crate::lexer::LexerRule {
            pattern: TokenPattern::new(r"\w+", Token::NAME).unwrap(),
            action: LexerAction::Token(Token::NAME),
        });
        lang.add_rule("root", crate::lexer::LexerRule {
            pattern: TokenPattern::new(r"[^\w]+", Token::OTHER).unwrap(),
            action: LexerAction::Token(Token::OTHER),
        });

        let delegating = DelegatingLexer::new(
            "template",
            Box::new(root),
            Box::new(lang),
        );

        let tokens = delegating.tokenize("<div> if return </div>");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::NAME_TAG));
        assert!(types.contains(&Token::KEYWORD));
        assert!(types.contains(&Token::NAME));
    }

    #[test]
    fn test_using_this() {
        let mut lexer = ExtendedRegexLexer::new("using_self_test");

        // Match a parenthesized expression and re-tokenize with self
        lexer.add_rule("root", ExtendedRule {
            pattern: TokenPattern::new(r"\(([^)]+)\)", Token::TEXT).unwrap(),
            action: using_this(),
            new_state: None,
        });
        lexer.add_rule("root", rule_token_push(r"\w+", Token::NAME, None).unwrap());
        lexer.add_rule("root", rule_token_push(r"[()]", Token::PUNCTUATION, None).unwrap());

        // Simple case without recursion
        let tokens = lexer.tokenize("foo bar");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::NAME));
    }

    #[test]
    fn test_using_factory() {
        let mut factory = RegistryFactory::new();

        // Register a simple lexer using a concrete type
        factory.register("sub", || {
            let mut sub = crate::lexer::RegexLexer::new("sub");
            sub.add_rule("root", crate::lexer::LexerRule {
                pattern: TokenPattern::new(r"\d+", Token::NUMBER).unwrap(),
                action: LexerAction::Token(Token::NUMBER),
            });
            sub.add_rule("root", crate::lexer::LexerRule {
                pattern: TokenPattern::new(r"[^\d]+", Token::TEXT).unwrap(),
                action: LexerAction::Token(Token::TEXT),
            });
            Box::new(sub)
        });

        let mut lexer = ExtendedRegexLexer::with_factory("main", Arc::new(factory));

        lexer.add_rule("root", ExtendedRule {
            pattern: TokenPattern::new(r"\{([^}]+)\}", Token::TEXT).unwrap(),
            action: using("sub"),
            new_state: None,
        });
        lexer.add_rule("root", rule_token_push(r"\w+", Token::NAME, None).unwrap());

        let tokens = lexer.tokenize("hello {42} world");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&Token::NAME));
        assert!(types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_combined_state() {
        let mut lexer = ExtendedRegexLexer::new("combined_test");

        lexer.add_rule("strings", rule_token_push(r#""[^"]*""#, Token::STRING, None).unwrap());
        lexer.add_rule("numbers", rule_token_push(r"\d+", Token::NUMBER, None).unwrap());

        // Combined state merges both
        lexer.add_combined_state("string_or_num", &["strings", "numbers"]);

        let tokens = lexer.tokenize("hello");
        // No match in combined state for "hello"
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_eol_reset() {
        let mut lexer = ExtendedRegexLexer::new("eol_test");

        lexer.add_rule("root", ExtendedRule {
            pattern: TokenPattern::new(r"<", Token::PUNCTUATION).unwrap(),
            action: ExtendedAction::Token(Token::PUNCTUATION),
            new_state: Some(ExtendedState::Push("tag".to_string())),
        });
        lexer.add_rule("tag", rule_token_push(r"\w+", Token::NAME, None).unwrap());
        // No \n rule in tag state — EOL should trigger reset to root

        // Newline in tag state (no matching rule) should reset to root
        let tokens = lexer.tokenize("<\nfoo");
        let types: Vec<Token> = tokens.iter().map(|t| t.token_type).collect();
        assert_eq!(types[0], Token::PUNCTUATION); // <
        assert_eq!(types[1], Token::WHITESPACE);  // \n (EOL reset to root)
    }
}
