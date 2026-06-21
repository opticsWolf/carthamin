use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;
use crate::unistring::{XID_START, XID_CONTINUE};

/// Scala lexer for Scala source code.
pub struct ScalaLexer {
    inner: RegexLexer,
}

impl ScalaLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("Scala");
        inner.aliases = vec!["scala"];
        inner.filenames = vec!["*.scala"];
        inner.mimetypes = vec!["text/x-scala"];

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Whitespace and newlines
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) });

        // Comments
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(?s)/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING_CHAR).unwrap(), action: LexerAction::token(Token::STRING_CHAR) });

        // Raw triple-quoted strings — FIXED: use r## to embed quotes, single backslashes
        root_rules.push(LexerRule { pattern: TokenPattern::new(r##""""".*?"""##, Token::STRING_DOUBLE).unwrap(), action: LexerAction::token(Token::STRING_DOUBLE) });

        // Numbers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?[fFdDlL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Operators — FIXED: use single backslashes in raw string
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|=>|<:|<%|>:|&&|!=|==|<=|>=|<|>|!|~|\||\?|:|=|\+|-|\*|/|%|&|\^", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[()\[\]{};:,.\@\#]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Keywords
        let keywords = [
            "abstract", "case", "class", "def", "do", "else", "enum", "export",
            "extends", "final", "finally", "for", "if", "implicit",
            "lazy", "match", "new", "null", "override", "package",
            "private", "protected", "return", "sealed", "super", "this", "throw",
            "try", "type", "val", "var", "while", "with", "yield",
            "catch", "clone", "const", "false", "for", "fun", "function",
            "import", "infix", "inline", "inner", "interface", "internal", "is",
            "object", "operator", "out", "public", "reified", "sealed",
            "super", "synchronized", "tailrec", "throws", "transient", "true",
            "typealias", "value", "vararg", "when", "where",
        ];
        let keyword_pattern = format!(r"\b({})\b", keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&keyword_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Type keywords
        let type_keywords = [
            "Any", "Array", "Boolean", "Byte", "Char", "Double",
            "Float", "Int", "Long", "Nothing", "Short", "String", "Unit",
        ];
        let type_pattern = format!(r"\b({})\b", type_keywords.join("|"));
        root_rules.push(LexerRule { pattern: TokenPattern::new(&type_pattern, Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Identifiers
        // Identifiers — Unicode-aware via XID_START/XID_CONTINUE
        let ident_pattern = format!("[{}][{}]*", XID_START, XID_CONTINUE);
        root_rules.push(LexerRule { pattern: TokenPattern::new(&ident_pattern, Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        ScalaLexer { inner }
    }
}

impl Lexer for ScalaLexer {
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
    fn test_scala_keywords() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("class Hello { def main(args: Array[String]) { } }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_scala_comment() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("// comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    // --- Pattern matching ---

    #[test]
    fn test_scala_pattern_match() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens(
            "x match {
                case 1 => \"one\"
                case \"hello\" => \"greeting\"
                case _ => \"default\"
            }",
        );
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // match, case
        assert!(token_types.contains(&Token::STRING_DOUBLE));
        assert!(token_types.contains(&Token::OPERATOR)); // =>
    }

    #[test]
    fn test_scala_pattern_match_guard() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("x match { case y if y > 0 => y }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // match, case, if
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_scala_pattern_match_or() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("x match { case 1 | 2 | 3 => \"small\" }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
        assert!(token_types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_scala_pattern_match_wildcard() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("x match { case _ => \"anything\" }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // match, case
    }

    // --- String interpolation ---

    #[test]
    fn test_scala_string_interpolation() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens(r#"s"Hello, $name!""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        // The 's' prefix and string content are both tokenized
        assert!(token_types.contains(&Token::NAME)); // s
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_scala_f_string_interpolation() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens(r#"f"Value: $value%2.2f""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME)); // f
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_scala_raw_string() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens(r#"raw"""multi
line
string""""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME)); // raw
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    // --- Triple-quoted strings ---

    #[test]
    fn test_scala_triple_quoted_string() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens(r#"""""line1
line2
line3""""#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    // --- Implicit declarations ---

    #[test]
    fn test_scala_implicit() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("implicit val converter: Int => String = _.toString");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // implicit, val
        assert!(token_types.contains(&Token::OPERATOR)); // =>
    }

    #[test]
    fn test_scala_implicit_class() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("implicit class RichInt(val x: Int) {
    def isEven: Boolean = x % 2 == 0
}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // implicit, class, val, def
    }

    // --- Import statements ---

    #[test]
    fn test_scala_import() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("import scala.collection.immutable._");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // import
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_scala_import_rename() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("import scala.util.{Try, Failure => F}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // import
        assert!(token_types.contains(&Token::OPERATOR)); // =>
    }

    // --- Case classes ---

    #[test]
    fn test_scala_case_class() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("case class Person(name: String, age: Int)");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // case, class
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_scala_case_object() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("case object Singleton {
    val value = 42
}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // case, object, val
    }

    // --- Objects and traits ---

    #[test]
    fn test_scala_object() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("object MyApp {\n    def main(args: Array[String]): Unit = {\n        println(\"Hello\")\n    }\n}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // object, def
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    #[test]
    fn test_scala_trait() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("trait Printable {
    def print(): Unit
    def format(indent: Int): String
}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // trait, def
    }

    // --- Extends / with ---

    #[test]
    fn test_scala_extends() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("class Dog extends Animal with Printable {
    override def speak(): String = \"woof\"
}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // class, extends, with, override, def
    }

    // --- For comprehensions ---

    #[test]
    fn test_scala_for_comprehension() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("for {
    x <- List(1, 2, 3)
    y <- List(4, 5)
} yield x + y");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // for, yield
        assert!(token_types.contains(&Token::OPERATOR)); // <-
    }

    // --- Type annotations ---

    #[test]
    fn test_scala_type_annotation() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("val items: Map[String, List[Int]] = Map.empty");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // val
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Numbers ---

    #[test]
    fn test_scala_numbers() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("val i = 42; val d = 3.14; val s = 100L; val f = 1.5f");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
        assert!(token_types.contains(&Token::KEYWORD)); // val
    }

    // --- Operators ---

    #[test]
    fn test_scala_operators() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("a -> b => c <: d >: e && f != g");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR));
    }

    // --- Multi-line comments ---

    #[test]
    fn test_scala_multiline_comment() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("/* this is\na multi-line\ncomment */");
        assert_eq!(tokens[0].0, Token::COMMENT_MULTILINE);
    }

    // --- SBT-style syntax ---

    #[test]
    fn test_scala_sbt_syntax() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("name := \"my-project\"\nversion := \"1.0\"\nscalaVersion := \"2.13.8\"");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME));
        assert!(token_types.contains(&Token::OPERATOR)); // :=
        assert!(token_types.contains(&Token::STRING_DOUBLE));
    }

    // --- Sealed classes ---

    #[test]
    fn test_scala_sealed() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("sealed trait Result[+T]\ncase class Success[T](value: T) extends Result[T]\ncase class Failure(error: String) extends Result[Nothing]");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // sealed, trait, case, class, extends
    }

    // --- Lambda / anonymous function ---

    #[test]
    fn test_scala_lambda() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("val add: (Int, Int) => Int = (a, b) => a + b");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // val
        assert!(token_types.contains(&Token::OPERATOR)); // =>
    }

    // --- Try/catch/finally ---

    #[test]
    fn test_scala_try_catch() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("try {\n    riskyOperation()\n} catch {\n    case e: Exception => handle(e)\n} finally {\n    cleanup()\n}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // try, catch, case, finally
    }

    // --- While / do-while ---

    #[test]
    fn test_scala_while() {
        let lexer = ScalaLexer::new();
        let tokens = lexer.get_tokens("while (i < 10) {\n    i += 1\n}");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // while
    }

    // --- Round-trip reconstruction ---

    #[test]
    fn test_scala_roundtrip() {
        let lexer = ScalaLexer::new();
        let source = "class Hello { def main(args: Array[String]) { println(\"world\") } }";
        let tokens = lexer.get_tokens(source);
        let reconstructed: String = tokens.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(reconstructed, source);
    }
}
