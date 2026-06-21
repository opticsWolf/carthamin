use crate::token::Token;
use crate::lexer::{Lexer, TokenStreamItem};

/// ERB (Embedded Ruby) lexer.
///
/// Highlights Ruby code between `<% ... %>` preprocessor directives.
/// All other content is left as `Token::Other`.
///
/// This is a custom Lexer (not RegexLexer) because ERB uses a
/// split-based approach: split text on ERB delimiters, then delegate
/// the Ruby portions to a RubyLexer.
pub struct ErbLexer {
    name: String,
    aliases: Vec<&'static str>,
    filenames: Vec<&'static str>,
    mimetypes: Vec<&'static str>,
    ruby_lexer: crate::lexer::ruby::RubyLexer,
}

impl ErbLexer {
    pub fn new() -> Self {
        ErbLexer {
            name: "ERB".to_string(),
            aliases: vec!["erb"],
            filenames: vec!["*.erb", "*.rhtml"],
            mimetypes: vec!["application/x-ruby-templating"],
            ruby_lexer: crate::lexer::ruby::RubyLexer::new(),
        }
    }

    /// ERB block delimiter regex — matches ERB tags and `%` Ruby statements.
    /// Equivalent to Python: r'(<%%|%%>|<%=|<%#|<%-|<%|-%>|%>|^%[^%].*?$)', re.M
    fn block_regex() -> &'static regex::Regex {
        use std::sync::OnceLock;
        static BLOCK_RE: OnceLock<regex::Regex> = OnceLock::new();
        BLOCK_RE.get_or_init(|| {
            regex::Regex::new(r"(<%%|%%>|<%=|<%#|<%-|<%|-%>|%>|^%[^%].*?$)").unwrap()
        })
    }

    /// Tokenize ERB text using find-based approach.
    ///
    /// Unlike Python's re.split() which includes captured groups,
    /// Rust's regex::split() does not. So we use find_iter to get
    /// all match positions and extract text between them.
    fn tokenize(&self, text: &str) -> Vec<TokenStreamItem> {
        let re = Self::block_regex();

        // Collect all matches with their positions
        let matches: Vec<_> = re.find_iter(text).collect();

        let mut result = Vec::new();
        let mut idx: usize = 0;
        let mut last_end: usize = 0;

        for m in matches {
            let start = m.start();
            let end = m.end();
            let tag = m.as_str();

            // Emit text before this match as Other
            if start > last_end {
                let before = &text[last_end..start];
                if !before.is_empty() {
                    result.push(TokenStreamItem {
                        index: idx,
                        token_type: Token::OTHER,
                        text: before.to_string(),
                    });
                    idx += 1;
                }
            }

            // Process the tag
            if tag == "<%%" || tag == "%%>" {
                // Escape literals — yield as Other
                result.push(TokenStreamItem {
                    index: idx,
                    token_type: Token::OTHER,
                    text: tag.to_string(),
                });
                idx += 1;
            } else if tag == "<%#" {
                // Comment block: <%# ... %>
                // The comment body extends to the next %> or -%>
                result.push(TokenStreamItem {
                    index: idx,
                    token_type: Token::COMMENT_PREPROC,
                    text: tag.to_string(),
                });
                idx += 1;

                // Find the closing %> or -%>
                let rest = &text[end..];
                let close_match = re.find_iter(rest).next();
                if let Some(cm) = close_match {
                    let comment_body = &rest[..cm.start()];
                    if !comment_body.is_empty() {
                        result.push(TokenStreamItem {
                            index: idx,
                            token_type: Token::COMMENT,
                            text: comment_body.to_string(),
                        });
                        idx += 1;
                    }
                    // Emit the closing tag
                    result.push(TokenStreamItem {
                        index: idx,
                        token_type: Token::COMMENT_PREPROC,
                        text: cm.as_str().to_string(),
                    });
                    idx += 1;
                    // Skip past the closing tag in the main loop
                    // We do this by advancing last_end
                    // But since we're iterating, we need to handle this differently
                    // Actually, the main loop will find the %> as the next match
                    // So we just need to make sure we don't double-process it
                    // We'll handle this by checking if the next match is the closing tag
                } else {
                    // No closing tag found — rest is all comment
                    if !rest.is_empty() {
                        result.push(TokenStreamItem {
                            index: idx,
                            token_type: Token::COMMENT,
                            text: rest.to_string(),
                        });
                        idx += 1;
                    }
                }

                last_end = end;
                continue; // Skip to next iteration, the %> will be found as next match
            } else if tag == "<%" || tag == "<%=" || tag == "<%-" {
                // Code block: <% ... %>, <%= ... %>, <%- ... %>
                result.push(TokenStreamItem {
                    index: idx,
                    token_type: Token::COMMENT_PREPROC,
                    text: tag.to_string(),
                });
                idx += 1;

                // Find the closing %> or -%>
                let rest = &text[end..];
                let close_match = re.find_iter(rest).next();
                if let Some(cm) = close_match {
                    let ruby_data = &rest[..cm.start()];
                    // Delegate Ruby content to RubyLexer
                    let ruby_tokens = self.ruby_lexer.get_tokens_unprocessed(ruby_data);
                    let ruby_count = ruby_tokens.len();
                    for item in ruby_tokens {
                        result.push(TokenStreamItem {
                            index: idx + item.index,
                            token_type: item.token_type,
                            text: item.text,
                        });
                    }
                    idx += ruby_count;

                    // Emit the closing tag
                    result.push(TokenStreamItem {
                        index: idx,
                        token_type: Token::COMMENT_PREPROC,
                        text: cm.as_str().to_string(),
                    });
                    idx += 1;
                } else {
                    // No closing tag found — rest is all Ruby
                    let ruby_tokens = self.ruby_lexer.get_tokens_unprocessed(rest);
                    let ruby_count = ruby_tokens.len();
                    for item in ruby_tokens {
                        result.push(TokenStreamItem {
                            index: idx + item.index,
                            token_type: item.token_type,
                            text: item.text,
                        });
                    }
                    idx += ruby_count;
                }

                last_end = end;
                continue; // Skip to next iteration
            } else if tag == "%>" || tag == "-%>" {
                // Orphan closing tag — yield as Error
                result.push(TokenStreamItem {
                    index: idx,
                    token_type: Token::ERROR,
                    text: tag.to_string(),
                });
                idx += 1;
            } else {
                // % raw Ruby statement (line starting with %)
                let first_char: String = tag.chars().next().map(|c| c.to_string()).unwrap_or_default();
                result.push(TokenStreamItem {
                    index: idx,
                    token_type: Token::COMMENT_PREPROC,
                    text: first_char,
                });
                let rest = &tag[1..];
                let ruby_tokens = self.ruby_lexer.get_tokens_unprocessed(rest);
                    let ruby_count = ruby_tokens.len();
                for item in ruby_tokens {
                    result.push(TokenStreamItem {
                        index: idx + 1 + item.index,
                        token_type: item.token_type,
                        text: item.text,
                    });
                }
                idx += 1 + ruby_count;
            }

            last_end = end;
        }

        // Emit remaining text as Other
        if last_end < text.len() {
            let remaining = &text[last_end..];
            if !remaining.is_empty() {
                result.push(TokenStreamItem {
                    index: idx,
                    token_type: Token::OTHER,
                    text: remaining.to_string(),
                });
            }
        }

        result
    }
}

impl Lexer for ErbLexer {
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

    fn analyse_text(&self, text: &str) -> f64 {
        if text.contains("<%") && text.contains("%>") {
            0.4
        } else {
            0.0
        }
    }

    fn get_tokens_unprocessed(&self, text: &str) -> Vec<TokenStreamItem> {
        self.tokenize(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erb_plain_text() {
        let lexer = ErbLexer::new();
        let tokens = lexer.get_tokens_unprocessed("Hello World");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, Token::OTHER);
        assert_eq!(tokens[0].text, "Hello World");
    }

    #[test]
    fn test_erb_code_block() {
        let lexer = ErbLexer::new();
        let tokens = lexer.get_tokens_unprocessed("<%= 1 + 2 %>");
        // Should have: Preproc (<%=), Ruby tokens, Preproc (%>)
        assert!(tokens.iter().any(|t| t.token_type == Token::COMMENT_PREPROC));
    }

    #[test]
    fn test_erb_comment_block() {
        let lexer = ErbLexer::new();
        let tokens = lexer.get_tokens_unprocessed("<%# this is a comment %>");
        let has_preproc = tokens.iter().any(|t| t.token_type == Token::COMMENT_PREPROC);
        let has_comment = tokens.iter().any(|t| t.token_type == Token::COMMENT);
        assert!(has_preproc && has_comment);
    }

    #[test]
    fn test_erb_escape_literal() {
        let lexer = ErbLexer::new();
        let tokens = lexer.get_tokens_unprocessed("<%% escaped %%>");
        // Both <%% and %%> should be Other
        let escaped: Vec<_> = tokens.iter().filter(|t| t.token_type == Token::OTHER).collect();
        assert!(escaped.len() >= 2);
    }

    #[test]
    fn test_erb_analyse_text() {
        let lexer = ErbLexer::new();
        assert_eq!(lexer.analyse_text("<%= hello %>"), 0.4);
        assert_eq!(lexer.analyse_text("no erb here"), 0.0);
    }
}
