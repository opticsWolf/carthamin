use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// Makefile lexer for Makefile syntax.
pub struct MakefileLexer {
    inner: RegexLexer,
}

impl MakefileLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Makefile");
        inner.aliases = vec!["makefile", "make"];
        inner.filenames = vec!["Makefile", "makefile", "*.mk"];
        inner.mimetypes = vec!["text/x-makefile"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Comments (before whitespace so # at start of line is caught)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"#[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });

        // Whitespace and tabs
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[ \t]+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\n", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Variables
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\([^\)]*\)", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$\{[^\}]*\}", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\$[\w@<>*?^%+]", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Phony targets and special targets - must come before generic target rule
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.PHONY\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.SUFFIXES\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.DEFAULT\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Directives
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(export|include|sinclude|-include|define|endef|ifdef|ifndef|ifeq|ifneq|else|endif|vpath)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"::=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r":=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\?=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\+=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"!=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"=", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Targets (name followed by colon)
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z0-9_./%-]+:", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Identifiers (variable names, values, filenames) - before punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z0-9_./%-]+", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[;@|]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Functions
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(word|words|firstword|lastword|wordlist|filter|filter-out|sort|findstring|foreach|if|or|and|call|eval|value|origin|flavor|shell|strip|patsubst|subst|realpath|abspath|dir|notdir|suffix|basename|addsuffix|addprefix|join|reverse|unique|error|warning|info)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        MakefileLexer { inner }
    }
}

impl Lexer for MakefileLexer {
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

    // --- Basic tests ---

    #[test]
    fn test_makefile_basic() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("all:\n\techo hello");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_comment() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("# comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    // --- Variables ---

    #[test]
    fn test_makefile_variable_simple() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("CC = gcc");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_makefile_variable_export() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("export CFLAGS = -O2 -Wall");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Variable expansion ---

    #[test]
    fn test_makefile_variable_paren() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("$(CC) $(CFLAGS)");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_variable_brace() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("${CC} ${CFLAGS}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_variable_single() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("$@ $< $*");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Assignment operators ---

    #[test]
    fn test_makefile_assign_simple() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("VAR = value");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_makefile_assign_recursive() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("VAR := value");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_makefile_assign_conditional() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("VAR ?= default");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_makefile_assign_append() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("CFLAGS += -Wextra");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR));
    }

    // --- Targets ---

    #[test]
    fn test_makefile_target() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("all: main.o utils.o\n\t$(CC) -o main main.o utils.o");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_target_double_colon() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("clean::\n\trm -f *.o");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- .PHONY ---

    #[test]
    fn test_makefile_phony() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens(".PHONY: all clean install");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    // --- VPATH ---

    #[test]
    fn test_makefile_vpath() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("VPATH = src:include");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_vpath_clear() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("vpath % .");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Automake variables ---

    #[test]
    fn test_makefile_automake_bins() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("bin_PROGRAMS = myapp\nmyapp_SOURCES = main.c utils.c");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_makefile_automake_ldadd() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("myapp_LDADD = $(LIBS)");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- BSD make features ---

    #[test]
    fn test_makefile_bsd_msg() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens(".BEGIN:\n\t@echo Building...");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Functions ---

    #[test]
    fn test_makefile_function_patsubst() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("OBJS = $(patsubst %.c,%.o,$(SRCS))");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_function_shell() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("FILES = $(shell find . -name \"*.c\")");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_function_if() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("$(if $(DEBUG),-g,-O2)");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_function_wildcard() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("SRCS = $(wildcard src/*.c)");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Conditional directives ---

    #[test]
    fn test_makefile_ifdef() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("ifdef DEBUG\nCFLAGS += -g\nendif");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_ifeq() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("ifeq ($(OS),Linux)\nCC = gcc\nendif");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Special targets ---

    #[test]
    fn test_makefile_suffixes() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens(".SUFFIXES: .c .o");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    // --- Recipe lines ---

    #[test]
    fn test_makefile_recipe() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("main.o: main.c\n\t$(CC) -c main.c -o main.o");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_makefile_recipe_silent() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("all:\n\t@echo Done");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::PUNCTUATION)); // @
    }

    // --- Automatic variables ---

    #[test]
    fn test_makefile_automatic_vars() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("%.o: %.c\n\t$(CC) -c $< -o $@");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Include ---

    #[test]
    fn test_makefile_include() {
        let lexer = MakefileLexer::new();
        let tokens = lexer.get_tokens("include config.mk");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Round-trip reconstruction ---

    #[test]
    fn test_makefile_roundtrip() {
        let lexer = MakefileLexer::new();
        let source = "all:\n\techo hello";
        let tokens = lexer.get_tokens(source);
        let reconstructed: String = tokens.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(reconstructed, source);
    }
}
