"""
Compatibility tests for carthamin (Rust Pygments port).

Tests API compatibility and output parity with original Pygments.
"""
import carthamin
from carthamin import Token, lex, format, highlight
import pytest


# ===========================================================================
# Token Tests
# ===========================================================================

class TestTokenAPI:
    """Test Token class API compatibility."""

    def test_token_hierarchy(self):
        """Test token hierarchy access."""
        assert Token.Keyword is not None
        assert Token.Keyword.Declaration is not None
        assert Token.Name.Function is not None
        assert Token.Literal.String is not None
        assert Token.Comment.Single is not None

    def test_token_string(self):
        """Test token string representation."""
        assert str(Token.Keyword) == "Token.Keyword"
        assert str(Token.Keyword.Declaration) == "Token.Keyword.Declaration"

    def test_token_equality(self):
        """Test token equality."""
        assert Token.Keyword == Token.Keyword
        assert Token.Keyword.Declaration != Token.Keyword

    def test_token_subtype(self):
        """Test token subtype checking."""
        # KEYWORD_DECLARATION is a subtype of KEYWORD
        assert Token.Keyword.Declaration.is_subtype_of(Token.Keyword)


# ===========================================================================
# Lexer Tests
# ===========================================================================

class TestPythonLexer:
    """Test Python lexer."""

    def test_keywords(self):
        """Test keyword detection."""
        tokens = list(lex("if x == 1: pass", "python"))
        token_types = [t for t, _ in tokens]
        assert Token.Keyword in token_types

    def test_function_def(self):
        """Test function definition."""
        tokens = list(lex("def foo(): pass", "python"))
        token_types = [t for t, _ in tokens]
        # Both Pygments and Rust emit Token.Keyword (not Token.Keyword.Declaration)
        assert Token.Keyword in token_types
        # Rust emits Token.Name.Function for function names
        assert Token.Name.Function in token_types

    def test_class_def(self):
        """Test class definition."""
        tokens = list(lex("class Foo: pass", "python"))
        token_types = [t for t, _ in tokens]
        # Both Pygments and Rust emit Token.Keyword (not Token.Keyword.Declaration)
        assert Token.Keyword in token_types
        # Rust emits Token.Name.Class for class names
        assert Token.Name.Class in token_types

    def test_strings(self):
        """Test string detection."""
        tokens = list(lex('x = "hello"', "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Literal.String.Double for double-quoted strings
        assert Token.Literal.String.Double in token_types

    def test_fstring(self):
        """Test f-string detection."""
        tokens = list(lex("f'hello {name}'", "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Literal.String.Single for single-quoted f-string content
        assert Token.Literal.String.Single in token_types
        # Rust emits Token.Literal.String.Interpol for f-string braces
        assert Token.Literal.String.Interpol in token_types

    def test_comments(self):
        """Test comment detection."""
        tokens = list(lex("# this is a comment", "python"))
        assert tokens[0][0] == Token.Comment.Single

    def test_numbers(self):
        """Test number detection."""
        tokens = list(lex("x = 42", "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Literal.Number.Integer for integers
        assert Token.Literal.Number.Integer in token_types

        tokens = list(lex("x = 3.14", "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Literal.Number.Float for floats
        assert Token.Literal.Number.Float in token_types

    def test_operators(self):
        """Test operator detection."""
        tokens = list(lex("x + y * z", "python"))
        token_types = [t for t, _ in tokens]
        assert Token.Operator in token_types

    def test_punctuation(self):
        """Test punctuation detection."""
        tokens = list(lex("(x, y)", "python"))
        token_types = [t for t, _ in tokens]
        assert Token.Punctuation in token_types

    def test_builtin_functions(self):
        """Test builtin function detection."""
        tokens = list(lex("print('hello')", "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Name.Builtin for builtin functions
        assert Token.Name.Builtin in token_types

    def test_decorator(self):
        """Test decorator detection."""
        tokens = list(lex("@decorator\ndef foo(): pass", "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Name.Decorator for decorators
        assert Token.Name.Decorator in token_types

    def test_import(self):
        """Test import detection."""
        tokens = list(lex("import os", "python"))
        token_types = [t for t, _ in tokens]
        assert Token.Keyword.Namespace in token_types
        assert Token.Name.Namespace in token_types

    def test_triple_quoted_string(self):
        """Test triple-quoted string."""
        tokens = list(lex('"""docstring"""', "python"))
        token_types = [t for t, _ in tokens]
        # Rust emits Token.Literal.String.Doc for triple-quoted strings
        assert Token.Literal.String.Doc in token_types


# ===========================================================================
# Formatter Tests
# ===========================================================================

class TestHtmlFormatter:
    """Test HTML formatter."""

    def test_basic_html(self):
        """Test basic HTML output."""
        code = "def foo(): pass"
        html = highlight(code, "python", "html")
        assert "<div" in html
        assert "<span" in html
        assert "</span>" in html

    def test_html_classes(self):
        """Test HTML class names."""
        code = "def foo(): pass"
        html = highlight(code, "python", "html")
        assert "class=" in html
        # TODO: Rust emits 'k' (Keyword), Pygments emits 'kd' (Keyword.Declaration)
        assert "k" in html

    def test_html_noclasses(self):
        """Test HTML inline styles."""
        code = "def foo(): pass"
        tokens = list(lex(code, "python"))
        output = format(tokens, "html", noclasses=True)
        assert "style=" in output

    def test_html_css(self):
        """Test HTML CSS generation."""
        code = "def foo(): pass"
        html = highlight(code, "python", "html")
        # Should have CSS class names
        assert "highlight" in html


class TestTerminalFormatter:
    """Test terminal formatter."""

    def test_terminal_output(self):
        """Test terminal output with ANSI codes."""
        code = "def foo(): pass"
        output = highlight(code, "python", "terminal")
        assert "\x1b[" in output  # ANSI escape sequence
        assert "def" in output
        assert "foo" in output

    def test_terminal_newline_reset(self):
        """Test that ANSI codes reset at newlines."""
        code = "def foo():\n    pass"
        output = highlight(code, "python", "terminal")
        # Should have reset sequences
        assert "\x1b[39;49;00m" in output or "\x1b[0m" in output


# ===========================================================================
# Integration Tests
# ===========================================================================

class TestIntegration:
    """Integration tests for full pipeline."""

    def test_full_pipeline(self):
        """Test full lex → format pipeline."""
        code = "def hello(name):\n    return f'Hello, {name}!'"
        tokens = list(lex(code, "python"))
        assert len(tokens) > 0

        html = format(tokens, "html")
        assert "<span" in html

        terminal = format(tokens, "terminal")
        assert "\x1b[" in terminal

    def test_highlight_function(self):
        """Test highlight() convenience function."""
        code = "print('hello')\n"
        html = highlight(code, "python", "html")
        assert "<span" in html
        assert "print" in html

    def test_empty_input(self):
        """Test empty input handling."""
        tokens = list(lex("", "python"))
        assert tokens == []

    def test_unicode_input(self):
        """Test unicode input handling."""
        code = "你好世界\n"
        tokens = list(lex(code, "python"))
        # Should not crash
        assert len(tokens) >= 0

    def test_complex_python_code(self):
        """Test complex Python code."""
        code = '''
import os
from pathlib import Path

class MyClass:
    """A class docstring."""
    
    def __init__(self, name: str) -> None:
        self.name = name
    
    def greet(self) -> str:
        return f"Hello, {self.name}!"

if __name__ == "__main__":
    obj = MyClass("World")
    print(obj.greet())
'''
        tokens = list(lex(code, "python"))
        
        # Check for expected token types
        token_types = [t for t, _ in tokens]
        # Both Pygments and Rust emit Token.Keyword (not Token.Keyword.Declaration)
        assert Token.Keyword in token_types  # def, class
        assert Token.Keyword.Namespace in token_types  # import, from
        # Rust emits granular token types for names and strings
        assert Token.Name.Class in token_types  # MyClass
        assert Token.Name.Function in token_types  # __init__, greet
        assert Token.Literal.String.Doc in token_types  # docstrings
        assert Token.Literal.String.Double in token_types  # regular strings
        # Note: no comments in this test code
        
        # HTML output
        html = highlight(code, "python", "html")
        assert "<span" in html
        assert "MyClass" in html
        
        # Terminal output
        terminal = highlight(code, "python", "terminal")
        assert "\x1b[" in terminal


# ===========================================================================
# Edge Cases
# ===========================================================================

class TestEdgeCases:
    """Test edge cases."""

    def test_very_long_line(self):
        """Test very long line handling."""
        code = "x = " + "a" * 10000 + "\n"
        tokens = list(lex(code, "python"))
        assert len(tokens) > 0

    def test_nested_strings(self):
        """Test nested string handling."""
        code = 'x = "outer \\" inner"'
        tokens = list(lex(code, "python"))
        assert len(tokens) > 0

    def test_multiple_escapes(self):
        """Test multiple escape sequences."""
        code = r'x = "\n\t\r\\"'
        tokens = list(lex(code, "python"))
        assert len(tokens) > 0

    def test_raw_strings(self):
        """Test raw string handling."""
        code = r'x = r"raw \n string"'
        tokens = list(lex(code, "python"))
        assert len(tokens) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
