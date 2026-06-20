"""
Style compatibility tests: verify Rust style output matches Pygments Python.

Tests that carthamin's generated styles produce identical CSS output
to the original Pygments styles.
"""
import pytest
import sys

# Ensure UTF-8 output on Windows
if sys.stdout.encoding and 'utf' not in sys.stdout.encoding.lower():
    sys.stdout.reconfigure(encoding='utf-8')

from pygments.styles import get_style_by_name as py_get_style, get_all_styles as py_get_all
from pygments.token import Token as PyToken
from carthamin import get_style_by_name as rs_get_style, get_all_styles as rs_get_all, Token


# ===========================================================================
# Style Registry Tests
# ===========================================================================

class TestStyleRegistry:
    """Test style registry compatibility."""

    def test_all_styles_match(self):
        """Test that all Pygments style names are available in Rust."""
        py_styles = set(py_get_all())
        rs_styles = set(rs_get_all())
        assert py_styles == rs_styles, (
            f"Style mismatch: missing={py_styles - rs_styles}, extra={rs_styles - py_styles}"
        )

    def test_get_style_by_name_exists(self):
        """Test that get_style_by_name works for known styles."""
        for name in ["monokai", "default", "dracula", "nord", "solarized-dark"]:
            style = rs_get_style(name)
            assert style.name == name

    def test_get_style_by_name_invalid(self):
        """Test that get_style_by_name raises for unknown styles."""
        with pytest.raises(ValueError, match="No style named"):
            rs_get_style("nonexistent_style_xyz")

    def test_style_background_color(self):
        """Test background colors match Pygments."""
        styles = {
            "monokai": "#272822",
            "default": "#f8f8f8",
            "dracula": "#282a36",
            "nord": "#2e3440",
            "solarized-dark": "#002b36",
            "solarized-light": "#fdf6e3",
            "gruvbox-dark": "#282828",
            "zenburn": "#3f3f3f",
        }
        for name, expected_bg in styles.items():
            rs_style = rs_get_style(name)
            actual_bg = (rs_style.background_color or "").lower()
            expected_bg_lower = expected_bg.lower()
            assert actual_bg == expected_bg_lower, (
                f"{name}: expected {expected_bg}, got {rs_style.background_color}"
            )


# ===========================================================================
# Style Color Tests
# ===========================================================================

# Token pairs: (name, pygments_token, rust_token)
TOKEN_PAIRS = [
    ("Keyword", PyToken.Keyword, Token.Keyword),
    ("Keyword.Declaration", PyToken.Keyword.Declaration, Token.Keyword.Declaration),
    ("Keyword.Type", PyToken.Keyword.Type, Token.Keyword.Type),
    ("String", PyToken.Literal.String, Token.Literal.String),
    ("String.Single", PyToken.Literal.String.Single, Token.Literal.String.Single),
    ("Comment", PyToken.Comment, Token.Comment),
    ("Comment.Single", PyToken.Comment.Single, Token.Comment.Single),
    ("Number", PyToken.Literal.Number, Token.Literal.Number),
    ("Number.Float", PyToken.Literal.Number.Float, Token.Literal.Number.Float),
    ("Name", PyToken.Name, Token.Name),
    ("Name.Class", PyToken.Name.Class, Token.Name.Class),
    ("Name.Function", PyToken.Name.Function, Token.Name.Function),
    ("Name.Builtin", PyToken.Name.Builtin, Token.Name.Builtin),
    ("Name.Variable", PyToken.Name.Variable, Token.Name.Variable),
    ("Operator", PyToken.Operator, Token.Operator),
    ("Punctuation", PyToken.Punctuation, Token.Punctuation),
    ("Generic.Heading", PyToken.Generic.Heading, Token.Generic.Heading),
    ("Generic.Subheading", PyToken.Generic.Subheading, Token.Generic.Subheading),
    ("Generic.Prompt", PyToken.Generic.Prompt, Token.Generic.Prompt),
    ("Error", PyToken.Error, Token.Error),
]


def _normalize_color(color):
    """Normalize color: strip '#' prefix for comparison."""
    if color and color.startswith("#"):
        return color[1:]
    return color


STYLES_TO_TEST = [
    "monokai", "default", "dracula", "nord", "solarized-dark",
    "solarized-light", "gruvbox-dark", "gruvbox-light", "zenburn",
    "one-dark", "github-dark", "material", "tango", "vim", "native",
]


class TestStyleColors:
    """Test that style colors match Pygments output."""

    @pytest.mark.parametrize("style_name", STYLES_TO_TEST)
    def test_all_tokens_match(self, style_name):
        """Test all token colors match for a given style."""
        py_style = py_get_style(style_name)
        rs_style = rs_get_style(style_name)

        mismatches = []
        for name, py_tok, rs_tok in TOKEN_PAIRS:
            py_s = py_style.style_for_token(py_tok)
            rs_s = rs_style.style_for_token(rs_tok)

            py_color = _normalize_color(py_s["color"])
            rs_color = _normalize_color(rs_s["color"])

            if py_color != rs_color:
                mismatches.append(f"{name}: py={py_color}, rs={rs_color}")

        if mismatches:
            pytest.fail(
                f"Style '{style_name}' has {len(mismatches)} color mismatches:\n" +
                "\n".join(f"  {m}" for m in mismatches)
            )


class TestStyleBoldItalic:
    """Test bold/italic/underline flags match Pygments output."""

    def test_default_style_bold(self):
        """Test bold flags for default style."""
        rs_style = rs_get_style("default")
        py_style = py_get_style("default")

        # Keyword should be bold in default
        rs_kw = rs_style.style_for_token(Token.Keyword)
        py_kw = py_style.style_for_token(PyToken.Keyword)
        assert rs_kw["bold"] == py_kw["bold"], (
            f"Keyword bold: py={py_kw['bold']}, rs={rs_kw['bold']}"
        )

    def test_monokai_no_bold(self):
        """Test that monokai has no bold by default."""
        rs_style = rs_get_style("monokai")
        py_style = py_get_style("monokai")

        rs_kw = rs_style.style_for_token(Token.Keyword)
        py_kw = py_style.style_for_token(PyToken.Keyword)
        assert rs_kw["bold"] == py_kw["bold"], (
            f"Keyword bold: py={py_kw['bold']}, rs={rs_kw['bold']}"
        )


# ===========================================================================
# Style Attributes Tests
# ===========================================================================

class TestStyleAttributes:
    """Test StyleAttributes class."""

    def test_attributes_access(self):
        """Test accessing style attributes."""
        rs_style = rs_get_style("monokai")
        attrs = rs_style.get_style_attributes(Token.Keyword)
        assert attrs.color == "#66d9ef"

    def test_attributes_css_string(self):
        """Test CSS string output."""
        rs_style = rs_get_style("monokai")
        attrs = rs_style.get_style_attributes(Token.Keyword)
        css = attrs.to_css_string()
        assert "color:#66d9ef" in css


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
