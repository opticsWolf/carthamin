#!/usr/bin/env python3
"""
Generator script to extract Pygments style definitions and emit Rust code.

Reads all style classes from pygments.styles._mapping.STYLES,
parses their explicit style definitions, and writes
carthamin-core/src/style/generated.rs with one builder function per style.

Usage:
    python generators/gen_styles.py
"""

import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
PROJECT_ROOT = Path(__file__).resolve().parent.parent
STYLE_OUT = PROJECT_ROOT / "carthamin-core" / "src" / "style" / "generated.rs"

# ---------------------------------------------------------------------------
# Token mapping: Python token repr → Rust Token constant
# ---------------------------------------------------------------------------
TOKEN_MAP: dict[str, str] = {
    "Token": "TOKEN",
    "Token.Text": "TEXT",
    "Token.Text.Whitespace": "WHITESPACE",
    "Token.Escape": "ESCAPE",
    "Token.Error": "ERROR",
    "Token.Other": "OTHER",

    "Token.Keyword": "KEYWORD",
    "Token.Keyword.Constant": "KEYWORD_CONSTANT",
    "Token.Keyword.Declaration": "KEYWORD_DECLARATION",
    "Token.Keyword.Namespace": "KEYWORD_NAMESPACE",
    "Token.Keyword.Pseudo": "KEYWORD_PSEUDO",
    "Token.Keyword.Reserved": "KEYWORD_RESERVED",
    "Token.Keyword.Type": "KEYWORD_TYPE",

    "Token.Name": "NAME",
    "Token.Name.Attribute": "NAME_ATTRIBUTE",
    "Token.Name.Builtin": "NAME_BUILTIN",
    "Token.Name.Builtin.Pseudo": "NAME_BUILTIN_PSEUDO",
    "Token.Name.Class": "NAME_CLASS",
    "Token.Name.Constant": "NAME_CONSTANT",
    "Token.Name.Decorator": "NAME_DECORATOR",
    "Token.Name.Entity": "NAME_ENTITY",
    "Token.Name.Exception": "NAME_EXCEPTION",
    "Token.Name.Function": "NAME_FUNCTION",
    "Token.Name.Function.Magic": "NAME_FUNCTION_MAGIC",
    "Token.Name.Property": "NAME_PROPERTY",
    "Token.Name.Label": "NAME_LABEL",
    "Token.Name.Namespace": "NAME_NAMESPACE",
    "Token.Name.Other": "NAME_OTHER",
    "Token.Name.Tag": "NAME_TAG",
    "Token.Name.Variable": "NAME_VARIABLE",
    "Token.Name.Variable.Class": "NAME_VARIABLE_CLASS",
    "Token.Name.Variable.Global": "NAME_VARIABLE_GLOBAL",
    "Token.Name.Variable.Instance": "NAME_VARIABLE_INSTANCE",
    "Token.Name.Variable.Magic": "NAME_VARIABLE_MAGIC",

    "Token.Literal": "LITERAL",
    "Token.Literal.Date": "LITERAL_DATE",

    "Token.Literal.String": "STRING",
    "Token.Literal.String.Affix": "STRING_AFFIX",
    "Token.Literal.String.Backtick": "STRING_BACKTICK",
    "Token.Literal.String.Char": "STRING_CHAR",
    "Token.Literal.String.Delimiter": "STRING_DELIMITER",
    "Token.Literal.String.Doc": "STRING_DOC",
    "Token.Literal.String.Double": "STRING_DOUBLE",
    "Token.Literal.String.Escape": "STRING_ESCAPE",
    "Token.Literal.String.Heredoc": "STRING_HEREDOC",
    "Token.Literal.String.Interpol": "STRING_INTERPOL",
    "Token.Literal.String.Other": "STRING_OTHER",
    "Token.Literal.String.Regex": "STRING_REGEX",
    "Token.Literal.String.Single": "STRING_SINGLE",
    "Token.Literal.String.Symbol": "STRING_SYMBOL",

    "Token.Literal.Number": "NUMBER",
    "Token.Literal.Number.Bin": "NUMBER_BIN",
    "Token.Literal.Number.Float": "NUMBER_FLOAT",
    "Token.Literal.Number.Hex": "NUMBER_HEX",
    "Token.Literal.Number.Integer": "NUMBER_INTEGER",
    "Token.Literal.Number.Integer.Long": "NUMBER_INTEGER_LONG",
    "Token.Literal.Number.Oct": "NUMBER_OCT",

    "Token.Operator": "OPERATOR",
    "Token.Operator.Word": "OPERATOR_WORD",

    "Token.Punctuation": "PUNCTUATION",
    "Token.Punctuation.Marker": "PUNCTUATION_MARKER",

    "Token.Comment": "COMMENT",
    "Token.Comment.Hashbang": "COMMENT_HASHBANG",
    "Token.Comment.Multiline": "COMMENT_MULTILINE",
    "Token.Comment.Preproc": "COMMENT_PREPROC",
    "Token.Comment.PreprocFile": "COMMENT_PREPROCFILE",
    "Token.Comment.Single": "COMMENT_SINGLE",
    "Token.Comment.Special": "COMMENT_SPECIAL",

    "Token.Generic": "GENERIC",
    "Token.Generic.Deleted": "GENERIC_DELETED",
    "Token.Generic.Emph": "GENERIC_EMPH",
    "Token.Generic.Error": "GENERIC_ERROR",
    "Token.Generic.Heading": "GENERIC_HEADING",
    "Token.Generic.Inserted": "GENERIC_INSERTED",
    "Token.Generic.Output": "GENERIC_OUTPUT",
    "Token.Generic.Prompt": "GENERIC_PROMPT",
    "Token.Generic.Strong": "GENERIC_STRONG",
    "Token.Generic.Subheading": "GENERIC_SUBHEADING",
    "Token.Generic.EmphStrong": "GENERIC_EMPH_STRONG",
    "Token.Generic.Traceback": "GENERIC_TRACEBACK",
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _token_repr(token) -> str:
    """Return the Python repr of a token, e.g. 'Token.Keyword.Declaration'."""
    return repr(token)


def _rust_token(token) -> str | None:
    """Map a Python token to the Rust Token::CONSTANT name, or None."""
    return TOKEN_MAP.get(_token_repr(token))


def _normalize_hex(color: str) -> str:
    """Normalize a hex color to #RRGGBB format."""
    c = color.lstrip("#")
    if len(c) == 3:
        c = c[0]*2 + c[1]*2 + c[2]*2
    return f"#{c}"


def _parse_style_string(style_str: str) -> str:
    """
    Parse a Pygments style string and convert to CSS format for Rust.

    Pygments style strings can contain (space-separated):
        - A hex color: #RRGGBB or #RGB
        - bg:#RRGGBB  (background color)
        - border:#RRGGBB  (border color — ignored in Rust)
        - bold / nobold
        - italic / noitalic
        - underline / nounderline
        - roman / sans / mono
        - noinherit

    Returns CSS string like "color:#ff0000;bg:#ffffff;bold:true;italic:true"
    or empty string if nothing to render.
    """
    parts: list[str] = []
    tokens = style_str.split()

    for token in tokens:
        if token.startswith("bg:"):
            color = token[3:]
            if color:
                parts.append(f"bg:{_normalize_hex(color)}")
        elif token.startswith("border:"):
            # Border not supported in Rust StyleAttributes — skip
            pass
        elif token == "bold":
            parts.append("bold:true")
        elif token == "nobold":
            parts.append("bold:false")
        elif token == "italic":
            parts.append("italic:true")
        elif token == "noitalic":
            parts.append("italic:false")
        elif token == "underline":
            parts.append("underline:true")
        elif token == "nounderline":
            parts.append("underline:false")
        elif token == "roman":
            parts.append("roman:true")
        elif token in ("sans", "mono", "noinherit"):
            pass  # Not mapped in Rust StyleAttributes
        elif token.startswith("#") or (len(token) >= 3 and all(c in "0123456789abcdefABCDEF" for c in token)):
            # It's a foreground color
            parts.append(f"color:{_normalize_hex(token)}")

    return ";".join(parts)


# ---------------------------------------------------------------------------
# Main extraction
# ---------------------------------------------------------------------------

def extract_styles():
    """
    Import all style classes from pygments, extract explicit style data,
    return list of (style_name, background, highlight, entries).

    entries: list of (rust_token_name, css_string) — only explicit defs.
    """
    from pygments.styles._mapping import STYLES

    results = []

    for class_name, (module, style_name, _) in sorted(STYLES.items(), key=lambda x: x[1][1]):
        mod = __import__(module, fromlist=[class_name])
        cls = getattr(mod, class_name)

        # Use cls.styles (the raw explicit definitions) — not _styles (resolved)
        entries = []
        for tok, style_str in cls.styles.items():
            rust_name = _rust_token(tok)
            if rust_name is None:
                # Unknown token (not in our Rust token set) — skip
                continue

            # Skip empty style strings (inherited tokens)
            if not style_str or style_str.strip() == "":
                continue

            css = _parse_style_string(style_str)
            if css:
                entries.append((rust_name, css))

        bg = getattr(cls, "background_color", "#ffffff") or "#ffffff"
        hl = getattr(cls, "highlight_color", "#ffffcc") or "#ffffcc"

        results.append((style_name, bg, hl, entries))

    return results


# ---------------------------------------------------------------------------
# Code generation
# ---------------------------------------------------------------------------

def _rust_fn_name(style_name: str) -> str:
    """Convert 'monokai' → 'monokai_style', 'github-dark' → 'github_dark_style'."""
    return style_name.replace("-", "_") + "_style"


def generate_rust(styles_data) -> str:
    """Generate the full Rust source for generated.rs."""

    lines: list[str] = []
    lines.append("// AUTO-GENERATED by generators/gen_styles.py — DO NOT EDIT BY HAND")
    lines.append("")
    lines.append("use crate::token::Token;")
    lines.append("use super::{Style, StyleAttributes};")
    lines.append("")

    # All style names
    lines.append("/// All generated style names (matches Pygments style aliases).")
    lines.append("pub const ALL_STYLE_NAMES: &[&str] = &[")
    for name, *_ in styles_data:
        lines.append(f'    "{name}",')
    lines.append("];")
    lines.append("")

    # One builder function per style
    for style_name, bg, hl, entries in styles_data:
        fn_name = _rust_fn_name(style_name)
        lines.append(f"/// Generated style: {style_name}")
        lines.append(f"pub fn {fn_name}() -> Style {{")
        lines.append(f'    let mut s = Style::new("{style_name}");')

        bg_hex = bg.lstrip("#")
        if len(bg_hex) == 3:
            bg_hex = bg_hex[0]*2 + bg_hex[1]*2 + bg_hex[2]*2
        lines.append(f'    s.default_style.bg = Some("#{bg_hex}".to_string());')
        lines.append("")

        if entries:
            lines.append("    let entries: &[(Token, &str)] = &[")
            for rust_tok, css in entries:
                lines.append(f'        (Token::{rust_tok}, "{css}"),')
            lines.append("    ];")
            lines.append("    for (token, css) in entries {")
            lines.append("        s.styles.insert(*token, StyleAttributes::from_css_string(css));")
            lines.append("    }")
            lines.append("")

        lines.append("    s")
        lines.append("}")
        lines.append("")

    # Registry: get style by name
    lines.append("/// Look up a generated style by name. Returns None if not found.")
    lines.append("pub fn get_style(name: &str) -> Option<Style> {")
    lines.append("    match name {")
    for style_name, *_ in styles_data:
        fn_name = _rust_fn_name(style_name)
        lines.append(f'        "{style_name}" => Some({fn_name}()),')
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main():
    print("Extracting Pygments style definitions...")
    styles_data = extract_styles()
    print(f"  Found {len(styles_data)} styles")

    total_entries = sum(len(d) for _, _, _, d in styles_data)
    print(f"  Total explicit entries: {total_entries}")

    # Print summary
    for name, bg, hl, entries in styles_data:
        print(f"  - {name:25s} bg={bg} entries={len(entries)}")

    rust_code = generate_rust(styles_data)

    STYLE_OUT.parent.mkdir(parents=True, exist_ok=True)
    STYLE_OUT.write_text(rust_code, encoding="utf-8")
    print(f"\nWrote {STYLE_OUT} ({len(rust_code)} bytes)")


if __name__ == "__main__":
    main()
