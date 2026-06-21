use crate::token::Token;
use crate::formatter::Formatter;

/// Output the text unchanged without any formatting.
pub struct NullFormatter;

impl Formatter for NullFormatter {
    fn name(&self) -> &str {
        "Text only"
    }

    fn extension(&self) -> &str {
        "txt"
    }

    fn mimetype(&self) -> &str {
        "text/plain"
    }

    fn format(
        &self,
        tokens: &[(Token, String)],
        outfile: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        for (_, value) in tokens {
            outfile.write_all(value.as_bytes())?;
        }
        Ok(())
    }
}

/// Format tokens as a raw representation for storing token streams.
///
/// Output format: ``tokentype<TAB>repr(tokenstring)\n`` per line.
/// The output can later be converted back to a token stream.
pub struct RawTokenFormatter {
    /// Optional ANSI color for highlighting error tokens.
    error_color: Option<&'static str>,
}

impl RawTokenFormatter {
    pub fn new() -> Self {
        Self { error_color: None }
    }

    pub fn with_error_color(color: &'static str) -> Self {
        Self { error_color: Some(color) }
    }
}

impl Default for RawTokenFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for RawTokenFormatter {
    fn name(&self) -> &str {
        "Raw tokens"
    }

    fn extension(&self) -> &str {
        "raw"
    }

    fn mimetype(&self) -> &str {
        "text/x-pygmentsraw"
    }

    fn format(
        &self,
        tokens: &[(Token, String)],
        outfile: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        for (ttype, value) in tokens {
            let line = format!("{:?}\t{:?}\n", ttype, value);
            outfile.write_all(line.as_bytes())?;
        }
        Ok(())
    }
}

/// Format tokens as appropriate for a new Pygments-style testcase.
///
/// Generates a Python test method with the source fragment and expected token list.
pub struct TestcaseFormatter;

impl Formatter for TestcaseFormatter {
    fn name(&self) -> &str {
        "Testcase"
    }

    fn extension(&self) -> &str {
        "py"
    }

    fn mimetype(&self) -> &str {
        "text/x-python"
    }

    fn format(
        &self,
        tokens: &[(Token, String)],
        outfile: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        // Collect raw source and formatted token lines
        let mut raw_source = String::new();
        let mut token_lines = String::new();
        let indent = "            ";

        for (ttype, value) in tokens {
            raw_source.push_str(value);
            token_lines.push_str(&format!(
                "{}({:?}, {:?}),\n",
                indent, ttype, value
            ));
        }

        let before = format!(
            "    def testNeedsName(lexer):\n        fragment = {:?}\n        tokens = [\n",
            raw_source
        );
        let after = "        ]\n        assert list(lexer.get_tokens(fragment)) == tokens\n";

        outfile.write_all(before.as_bytes())?;
        outfile.write_all(token_lines.as_bytes())?;
        outfile.write_all(after.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    fn sample_tokens() -> Vec<(Token, String)> {
        vec![
            (Token::NAME, "print".to_string()),
            (Token::PUNCTUATION, "(".to_string()),
            (Token::STRING, "\"hello\"".to_string()),
            (Token::PUNCTUATION, ")".to_string()),
        ]
    }

    #[test]
    fn test_null_formatter() {
        let formatter = NullFormatter;
        let tokens = sample_tokens();
        let mut output = Vec::<u8>::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "print(\"hello\")");
    }

    #[test]
    fn test_null_formatter_empty() {
        let formatter = NullFormatter;
        let tokens: Vec<(Token, String)> = vec![];
        let mut output = Vec::<u8>::new();
        formatter.format(&tokens, &mut output).unwrap();
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_raw_token_formatter() {
        let formatter = RawTokenFormatter::new();
        let tokens = sample_tokens();
        let mut output = Vec::<u8>::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("Name"));
        assert!(result.contains("print"));
        assert!(result.contains("String"));
        assert_eq!(result.lines().count(), 4);
    }

    #[test]
    fn test_testcase_formatter() {
        let formatter = TestcaseFormatter;
        let tokens = sample_tokens();
        let mut output = Vec::<u8>::new();
        formatter.format(&tokens, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("def testNeedsName(lexer)"));
        assert!(result.contains("fragment ="));
        assert!(result.contains("assert list(lexer.get_tokens(fragment)) == tokens"));
        assert!(result.contains("Name"));
    }
}
