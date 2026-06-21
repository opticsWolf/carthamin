use crate::token::Token;
use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};
use crate::scanner::TokenPattern;

/// C/C++ lexer supporting both C and C++ features.
pub struct CppLexer {
    inner: RegexLexer,
}

impl CppLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("C++");
        inner.aliases = vec!["cpp", "c++", "hpp", "cxx", "cc"];
        inner.filenames = vec!["*.cpp", "*.hpp", "*.cxx", "*.h", "*.cc", "*.hh"];
        inner.mimetypes = vec!["text/x-c++src", "text/x-c++hdr", "text/x-csrc", "text/x-chdr"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"(?s)/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        // Include comments and whitespace
        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Preprocessor directives
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"^#include\s*[<"].*?[>"]"#, Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#[a-zA-Z_]\w*\b", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#.*", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Numeric literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[bB][01_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[oO]?[0-7_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*\.[0-9_]+([eE][+-]?[0-9_]+)?[fF]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9_]+\.[0-9_]*([eE][+-]?[0-9_]+)?[fF]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|\.\.|::", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]=?|[<>]=?|<<=?|>>=?|&=|&&|\|=|\|\||\^=|\!|~", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(alignas|alignof|asm|attribute|bool|catch|class|concept|const_cast|constexpr|consteval|constinit|decltype|delete|dynamic_cast|explicit|export|friend|goto|ifconsteval|mutable|namespace|new|noexcept|nullptr|operator|override|requires|static_assert|static_cast|template|this|thread_local|throw|true|false|typeid|typename|try|using|virtual|co_await|co_return|co_yield)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Builtin types
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"\b(nullptr|NULL|stdin|stdout|stderr|true|false)\b", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Characters
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        CppLexer { inner }
    }
}

impl Lexer for CppLexer {
    fn get_tokens(&self, code: &str) -> Vec<(Token, String)> {
        self.inner.get_tokens(code)
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

/// C lexer (subset of C++).
pub struct CLexer {
    inner: RegexLexer,
}

impl CLexer {
    pub fn new() -> Self {
        let mut inner = RegexLexer::new("C");
        inner.aliases = vec!["c", "h"];
        inner.filenames = vec!["*.c", "*.h"];
        inner.mimetypes = vec!["text/x-csrc", "text/x-chdr"];

        // Comments and whitespace
        inner.states.insert("commentsandwhitespace".to_string(), vec![
            LexerRule { pattern: TokenPattern::new(r"\s+", Token::WHITESPACE).unwrap(), action: LexerAction::token(Token::WHITESPACE) },
            LexerRule { pattern: TokenPattern::new(r"//[^\n]*", Token::COMMENT_SINGLE).unwrap(), action: LexerAction::token(Token::COMMENT_SINGLE) },
            LexerRule { pattern: TokenPattern::new(r"(?s)/\*.*?\*/", Token::COMMENT_MULTILINE).unwrap(), action: LexerAction::token(Token::COMMENT_MULTILINE) },
        ]);

        // Root state
        let mut root_rules: Vec<LexerRule> = Vec::new();

        for rule in inner.states.get("commentsandwhitespace").cloned().unwrap_or_default() {
            root_rules.push(rule);
        }

        // Preprocessor directives
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#"^#include\s*[<"].*?[>"]"#, Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#[a-zA-Z_]\w*\b", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"^#.*", Token::COMMENT).unwrap(), action: LexerAction::token(Token::COMMENT) });

        // Numeric literals
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"0[xX][0-9a-fA-F_]+[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*[uU]?[lL]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[0-9][0-9_]*\.[0-9_]+([eE][+-]?[0-9_]+)?[fF]?", Token::NUMBER).unwrap(), action: LexerAction::token(Token::NUMBER) });

        // Punctuation
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[{}()\[\];,:]", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"->|\.", Token::PUNCTUATION).unwrap(), action: LexerAction::token(Token::PUNCTUATION) });

        // Operators
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[+\-*/%]=?|[<>]=?|<<=?|>>=?|&=|&&|\|=|\|\||\^=|\!", Token::OPERATOR).unwrap(), action: LexerAction::token(Token::OPERATOR) });

        // Keywords
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"(auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b", Token::KEYWORD).unwrap(), action: LexerAction::token(Token::KEYWORD) });

        // Strings
        root_rules.push(LexerRule { pattern: TokenPattern::new(r#""([^"\\]|\\.)*""#, Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Characters
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"'([^'\\]|\\.)*'", Token::STRING).unwrap(), action: LexerAction::token(Token::STRING) });

        // Identifiers
        root_rules.push(LexerRule { pattern: TokenPattern::new(r"[a-zA-Z_][a-zA-Z0-9_]*", Token::NAME).unwrap(), action: LexerAction::token(Token::NAME) });

        inner.states.insert("root".to_string(), root_rules);

        CLexer { inner }
    }
}

impl Lexer for CLexer {
    fn get_tokens(&self, code: &str) -> Vec<(Token, String)> {
        self.inner.get_tokens(code)
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
    fn test_cpp_basic() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("int main() { return 0; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD));
    }

    #[test]
    fn test_cpp_preprocessor() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("#include <iostream>");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }

    #[test]
    fn test_cpp_comment() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("// this is a comment");
        assert_eq!(tokens[0].0, Token::COMMENT_SINGLE);
    }

    #[test]
    fn test_cpp_string() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens(r#"std::cout << "hello";"#);
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::STRING));
    }

    #[test]
    fn test_c_basic() {
        let lexer = CLexer::new();
        let tokens = lexer.get_tokens("#include <stdio.h>\nint main() { return 0; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::COMMENT));
        assert!(token_types.contains(&Token::KEYWORD));
    }

    // --- Attributes ---

    #[test]
    fn test_cpp_attributes() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("[[nodiscard]] int compute();");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // [[nodiscard]] parsed as punctuation + name
        assert!(token_types.contains(&Token::NAME)); // nodiscard
    }

    #[test]
    fn test_cpp_attribute_deprecated() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("[[deprecated(\"Use new_func instead\")]] void old_func();");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NAME)); // deprecated
        assert!(token_types.contains(&Token::STRING));
    }

    // --- noexcept ---

    #[test]
    fn test_cpp_noexcept() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("void safe() noexcept;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // noexcept
    }

    #[test]
    fn test_cpp_noexcept_expr() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("void maybe() noexcept(true);");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // noexcept, true
    }

    // --- Namespace nesting ---

    #[test]
    fn test_cpp_namespace() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("namespace foo { namespace bar { void baz(); } }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // namespace
        assert!(token_types.contains(&Token::NAME));
    }

    #[test]
    fn test_cpp_nested_namespace() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("namespace a::b::c { int x; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // namespace
        assert!(token_types.contains(&Token::PUNCTUATION)); // ::
    }

    // --- Templates ---

    #[test]
    fn test_cpp_template() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("template<typename T> T identity(T x) { return x; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // template, typename
    }

    #[test]
    fn test_cpp_template_class() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("template<class T, class U>
class Pair {
    T first;
    U second;
};");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // template, class
    }

    // --- Lambda expressions ---

    #[test]
    fn test_cpp_lambda() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("auto f = [](int x) { return x * 2; };");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // auto, return
        assert!(token_types.contains(&Token::OPERATOR));
    }

    #[test]
    fn test_cpp_lambda_capture() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("auto f = [x, &y](int z) mutable { return x + y + z; };");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // auto, mutable, return
    }

    // --- constexpr ---

    #[test]
    fn test_cpp_constexpr() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("constexpr int max_val = 100;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // constexpr
        assert!(token_types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_cpp_constexpr_function() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("constexpr int square(int x) { return x * x; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // constexpr, return
    }

    // --- Classes ---

    #[test]
    fn test_cpp_class() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("class Foo : public Bar {
public:
    int x;
    void set(int v) { x = v; }
private:
    int y;
};");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // class, public, private, void
        assert!(token_types.contains(&Token::NAME));
    }

    // --- Struct ---

    #[test]
    fn test_cpp_struct() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("struct Point { int x; int y; };");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // struct, int
    }

    // --- Enums ---

    #[test]
    fn test_cpp_enum_class() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("enum class Color { Red, Green, Blue };");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // enum, class
    }

    // --- Multi-line comments ---

    #[test]
    fn test_cpp_multiline_comment() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("/* multi\nline\ncomment */");
        assert_eq!(tokens[0].0, Token::COMMENT_MULTILINE);
    }

    // --- Preprocessor directives ---

    #[test]
    fn test_cpp_define() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("#define MAX_SIZE 1024");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }

    #[test]
    fn test_cpp_ifdef() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("#ifdef DEBUG\n    printf(\"debug mode\");\n#endif");
        assert_eq!(tokens[0].0, Token::COMMENT);
    }

    // --- Numbers ---

    #[test]
    fn test_cpp_hex_number() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("int x = 0xFF;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_cpp_binary_number() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("int mask = 0b10101010;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }

    #[test]
    fn test_cpp_float() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("double pi = 3.14159; float rate = 0.05f;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::NUMBER));
    }

    // --- Operators ---

    #[test]
    fn test_cpp_operators() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("a += b; c && d; e || f; !g; ~h;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::OPERATOR));
    }

    // --- new/delete ---

    #[test]
    fn test_cpp_new_delete() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("int* p = new int(42); delete p;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // new, delete
    }

    // --- throw/try/catch ---

    #[test]
    fn test_cpp_throw() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("try { risky(); } catch (std::exception& e) { handle(e); }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // try, catch
    }

    // --- static_assert ---

    #[test]
    fn test_cpp_static_assert() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("static_assert(sizeof(int) == 4, \"int must be 4 bytes\");");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // static_assert, sizeof
    }

    // --- using / alias ---

    #[test]
    fn test_cpp_using() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("using namespace std; using Vec = std::vector<int>;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // using, namespace
    }

    // --- virtual / override ---

    #[test]
    fn test_cpp_virtual() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("class Base {
    virtual void foo() = 0;
};
class Derived : public Base {
    void foo() override { }
};");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // virtual, override, class, public, void
    }

    // --- auto / decltype ---

    #[test]
    fn test_cpp_auto() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("auto x = 42; auto&& ref = x;");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // auto
    }

    #[test]
    fn test_cpp_decltype() {
        let lexer = CppLexer::new();
        let tokens = lexer.get_tokens("decltype(auto) get() { return 42; }");
        let token_types: Vec<Token> = tokens.iter().map(|(t, _)| *t).collect();
        assert!(token_types.contains(&Token::KEYWORD)); // decltype, auto, return
    }

    // --- Round-trip reconstruction ---

    #[test]
    fn test_cpp_roundtrip() {
        let lexer = CppLexer::new();
        let source = "int main() { return 0; }";
        let tokens = lexer.get_tokens(source);
        let reconstructed: String = tokens.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(reconstructed, source);
    }
}
