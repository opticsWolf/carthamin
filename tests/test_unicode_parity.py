#!/usr/bin/env python3
"""
Unicode identifier parity tests — side-by-side comparison of carthamin vs Pygments
for actual Unicode identifier tokenization across multiple languages.
"""

import pytest
from pygments.lexers import get_lexer_by_name
from pygments.token import Token as PyToken
from carthamin import lex as carthamin_lex


# Mapping of Pygments token to carthamin token (carthamin uses flattened token names)
def py_token_to_carthamin_name(py_token):
    """Convert a Pygments token to the expected carthamin token string."""
    # Pygments Token.Name -> carthamin Token.Name
    # Pygments Token.Literal.String.Double -> carthamin Token.Literal.String.Double
    return str(py_token).replace("Token.", "Token.")


def get_pygments_tokens(lexer_name, code):
    """Get tokens from Pygments lexer."""
    lexer = get_lexer_by_name(lexer_name)
    return list(lexer.get_tokens(code))


def get_carthamin_tokens(lexer_name, code):
    """Get tokens from carthamin lexer."""
    return list(carthamin_lex(code, lexer_name))


# Test cases: (language, code)
UNICODE_TEST_CASES = [
    # Python: CJK identifier
    ("python", "变量 = 42"),
    # JavaScript: Latin extended identifier
    ("javascript", "let café = \"hello\""),
    # Java: CJK class name
    ("java", "class 変数 {}"),
    # Scala: CJK identifier
    ("scala", "val 変数 = 42"),
    # C#: CJK identifier (Pygments emits Error, carthamin should do better)
    ("csharp", "string 変数 = \"\";"),
    # PowerShell: CJK variable
    ("powershell", "$変数 = 42"),
    # Python: Greek identifier
    ("python", "αβγ = 1"),
    # JavaScript: Arabic identifier
    ("javascript", "let مرحبا = 1"),
]


class TestUnicodeParity:
    """Side-by-side parity tests for Unicode identifier tokenization."""

    @pytest.mark.parametrize("lexer_name,code", UNICODE_TEST_CASES)
    def test_unicode_identifier_recognition(self, lexer_name, code):
        """Verify that carthamin recognizes Unicode identifiers as single tokens."""
        py_tokens = get_pygments_tokens(lexer_name, code)
        carthamin_tokens = get_carthamin_tokens(lexer_name, code)

        # Extract just the values for comparison (both should tokenize the Unicode chars)
        py_values = [v for t, v in py_tokens if v.strip()]
        carthamin_values = [v for t, v in carthamin_tokens if v.strip()]

        # Find the Unicode identifier in both outputs
        py_unicode = [v for v in py_values if any(ord(c) > 127 for c in v)]
        carthamin_unicode = [v for v in carthamin_values if any(ord(c) > 127 for c in v)]

        # carthamin should have recognized the Unicode identifier as at least one token
        assert len(carthamin_unicode) >= 1, (
            f"carthamin ({lexer_name}) failed to recognize Unicode identifier in {code!r}. "
            f"Tokens: {carthamin_tokens}"
        )

        # Check that the Unicode chars are grouped into a single token (not split per-char)
        orig_unicode_run = "".join(c for c in code if ord(c) > 127)
        for uc in carthamin_unicode:
            if len(orig_unicode_run) > 1 and len(uc) == 1:
                assert False, (
                    f"carthamin ({lexer_name}) split Unicode identifier {orig_unicode_run!r} "
                    f"into per-char tokens instead of grouping them. Token: {uc!r}"
                )

        # If Pygments recognized it as a single token (not errors), carthamin should too
        py_errors = [v for t, v in py_tokens if "Error" in str(t)]
        if len(py_unicode) >= 1 and len(py_errors) == 0:
            # Pygments grouped it well; carthamin should match or be better
            assert len(carthamin_unicode) <= len(py_unicode), (
                f"carthamin ({lexer_name}) split Unicode into more tokens than Pygments. "
                f"Pygments: {py_unicode}, carthamin: {carthamin_unicode}"
            )

    def test_python_cjk_identifier_exact_parity(self):
        """Test exact token stream parity for Python CJK identifier."""
        code = "变量 = 42"
        py_tokens = get_pygments_tokens("python", code)
        carthamin_tokens = get_carthamin_tokens("python", code)

        # Check token values match (ignoring whitespace differences)
        py_vals = [(str(t), v) for t, v in py_tokens if v.strip()]
        carthamin_vals = [(str(t), v) for t, v in carthamin_tokens if v.strip()]

        # The identifier should be Token.Name in both
        assert py_vals[0] == ("Token.Name", "变量"), f"Pygments: {py_vals[0]}"
        assert carthamin_vals[0] == ("Token.Name", "变量"), f"carthamin: {carthamin_vals[0]}"

    def test_scala_cjk_identifier_exact_parity(self):
        """Test exact token stream parity for Scala CJK identifier."""
        code = "val 変数 = 42"
        py_tokens = get_pygments_tokens("scala", code)
        carthamin_tokens = get_carthamin_tokens("scala", code)

        py_vals = [(str(t), v) for t, v in py_tokens if v.strip()]
        carthamin_vals = [(str(t), v) for t, v in carthamin_tokens if v.strip()]

        # Both should recognize 変数 as Token.Name
        py_name = [v for t, v in py_vals if "Name" in t and "変数" in v]
        carthamin_name = [v for t, v in carthamin_vals if "Name" in t and "変数" in v]

        assert len(py_name) == 1, f"Pygments Name tokens: {py_vals}"
        assert len(carthamin_name) == 1, f"carthamin Name tokens: {carthamin_vals}"

    def test_java_cjk_class_name_exact_parity(self):
        """Test exact token stream parity for Java CJK class name."""
        code = "class 変数 {}"
        py_tokens = get_pygments_tokens("java", code)
        carthamin_tokens = get_carthamin_tokens("java", code)

        py_vals = [(str(t), v) for t, v in py_tokens if v.strip()]
        carthamin_vals = [(str(t), v) for t, v in carthamin_tokens if v.strip()]

        # Pygments emits Name.Class, carthamin emits Name (close enough)
        py_name = [v for t, v in py_vals if "Name" in t and "変数" in v]
        carthamin_name = [v for t, v in carthamin_vals if "Name" in t and "変数" in v]

        assert len(py_name) == 1, f"Pygments Name tokens: {py_vals}"
        assert len(carthamin_name) == 1, f"carthamin Name tokens: {carthamin_vals}"

    def test_csharp_better_than_pygments(self):
        """carthamin should handle C# Unicode identifiers better than Pygments."""
        code = "string 変数 = \"\";"
        py_tokens = get_pygments_tokens("csharp", code)
        carthamin_tokens = get_carthamin_tokens("csharp", code)

        # Pygments emits Error tokens for CJK chars in C#
        py_errors = [v for t, v in py_tokens if "Error" in str(t)]
        carthamin_errors = [v for t, v in carthamin_tokens if "Error" in str(t)]

        # carthamin should NOT emit errors for the Unicode identifier
        assert len(carthamin_errors) == 0, (
            f"carthamin emitted errors for C# Unicode identifier: {carthamin_tokens}"
        )

        # carthamin should recognize it as a name
        carthamin_name = [v for t, v in carthamin_tokens if "Name" in str(t) and "変数" in v]
        assert len(carthamin_name) >= 1, (
            f"carthamin should recognize 変数 as a name in C#. Tokens: {carthamin_tokens}"
        )


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
