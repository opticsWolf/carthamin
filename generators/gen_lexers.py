#!/usr/bin/env python3
"""
Lexer code generator for Carthamin.

Parses pygments lexer source files, extracts token definitions,
and generates Rust lexer implementations.

Usage:
    python generators/gen_lexers.py [--output DIR] [--lexer NAME] [--all]

Examples:
    python generators/gen_lexers.py                    # Generate all supported lexers
    python generators/gen_lexers.py --lexer python     # Generate only Python lexer
    python generators/gen_lexers.py --lexer bash       # Generate only Bash lexer
    python generators/gen_lexers.py --all              # Generate all (same as default)
    python generators/gen_lexers.py --output /tmp/out  # Output to different directory

Output:
    Each lexer generates a .rs file in carthamin-core/src/lexer/
    Plus a registry update in carthamin-core/src/registry.rs
"""

import argparse
import ast
import importlib
import inspect
import os
import re
import sys
from pathlib import Path
from typing import Any, Optional

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
PROJECT_ROOT = Path(__file__).resolve().parent.parent
LEXER_OUT_DIR = PROJECT_ROOT / "carthamin-core" / "src" / "lexer"
REGISTRY_FILE = PROJECT_ROOT / "carthamin-core" / "src" / "registry.rs"

# ---------------------------------------------------------------------------
# Token type mapping: Python repr → Rust constant
# ---------------------------------------------------------------------------

TOKEN_MAP: dict[str, str] = {
    # Base tokens
    "Token": "TOKEN",
    "Token.Text": "TEXT",
    "Token.Other": "OTHER",
    "Token.Error": "ERROR",
    "Token.Whitespace": "WHITESPACE",

    # Keywords
    "Token.Keyword": "KEYWORD",
    "Token.Keyword.Constant": "KEYWORD_CONSTANT",
    "Token.Keyword.Declaration": "KEYWORD_DECLARATION",
    "Token.Keyword.Namespace": "KEYWORD_NAMESPACE",
    "Token.Keyword.Pseudo": "KEYWORD_PSEUDO",
    "Token.Keyword.Reserved": "KEYWORD_RESERVED",
    "Token.Keyword.Type": "KEYWORD_TYPE",

    # Names
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

    # Literals
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

    # Operators
    "Token.Operator": "OPERATOR",
    "Token.Operator.Word": "OPERATOR_WORD",

    # Punctuation
    "Token.Punctuation": "PUNCTUATION",
    "Token.Punctuation.Marker": "PUNCTUATION_MARKER",

    # Comments
    "Token.Comment": "COMMENT",
    "Token.Comment.Hashbang": "COMMENT_HASHBANG",
    "Token.Comment.Multiline": "COMMENT_MULTILINE",
    "Token.Comment.Preproc": "COMMENT_PREPROC",
    "Token.Comment.PreprocFile": "COMMENT_PREPROCFILE",
    "Token.Comment.Single": "COMMENT_SINGLE",
    "Token.Comment.Special": "COMMENT_SPECIAL",

    # Generic
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
# Python token type → Rust Token constant
# ---------------------------------------------------------------------------

def python_token_to_rust(token_obj) -> Optional[str]:
    """
    Convert a Python token type object to a Rust Token::CONSTANT string.
    Handles Token types, None (for state transitions), and callables.
    """
    if token_obj is None:
        return None

    # If it's a callable (e.g., bygroups callback), skip
    if callable(token_obj):
        return None

    # Convert to string representation
    token_str = repr(token_obj)

    # Handle Token.Error
    if token_str == "Token.Error":
        return "ERROR"

    # Look up in mapping
    return TOKEN_MAP.get(token_str)


# ---------------------------------------------------------------------------
# Regex conversion: Python → Rust
# ---------------------------------------------------------------------------

def convert_regex(py_regex: str) -> str:
    r"""
    Convert a Python regex pattern to a Rust-compatible regex pattern.

    Most Python regex patterns are already compatible with Rust's regex crate.
    This handles known differences:
    - (?s) inline flag -> remove (Rust handles multiline differently)
    - Named capture groups -> unnamed (Rust doesn't support named groups)
    - Lookahead assertions -> kept (Rust supports them)
    - Unicode escapes -> kept (Rust supports \u{{HH}})
    """
    # Remove Python (?P<name>...) named groups -> make them non-capturing
    py_regex = re.sub(r'\(\?P<\w+>', '(?:', py_regex)

    # Remove (?s) inline flag (DOTALL) - Rust handles this differently
    py_regex = py_regex.replace('(?s)', '')

    # Remove (?m) inline flag if present (Rust handles differently)
    py_regex = py_regex.replace('(?m)', '')

    # Remove (?i) inline flag if present
    py_regex = py_regex.replace('(?i)', '')

    return py_regex


def rust_raw_string(pattern: str) -> str:
    """
    Return a Rust raw string literal for a regex pattern.

    Rust raw strings use r"...", r#"..."#, r##"..."## etc.
    The closing delimiter is the first occurrence of '"<n>#' where <n>
    matches the number of #s after r. So r#" ends at '"# and r##" ends at '"##.

    Key insight: a trailing " in the pattern followed by the delimiter #
    creates '"#' which prematurely closes the string. Use r##...## when
    the pattern ends with " or contains "#.

    Returns the pattern string with Rust raw string delimiters.
    """
    # Check if pattern ends with " - this creates '"#' with r# delimiter
    ends_with_dq = pattern.endswith('"')

    # Count consecutive #" sequences in the pattern
    max_hash_dq = 0
    current_hash_dq = 0
    for i, ch in enumerate(pattern):
        if ch == '#':
            if i + 1 < len(pattern) and pattern[i + 1] == '"':
                current_hash_dq += 1
                max_hash_dq = max(max_hash_dq, current_hash_dq)
            else:
                current_hash_dq = 0
        else:
            current_hash_dq = 0

    # Choose delimiter depth
    if ends_with_dq or max_hash_dq >= 1:
        # Pattern ends with " or has #" inside - need r##"..."##
        depth = max(2, max_hash_dq + 1)
        hashes = '#' * depth
        return f'r{hashes}"{pattern}"{hashes}'
    elif '"' in pattern:
        # Pattern has single ", use r#"..."#
        return f'r#"{pattern}"#'
    else:
        # Simple pattern, use r"..."
        return f'r"{pattern}"'


def rust_raw_string_for_code(pattern: str) -> str:
    """
    Return a Rust raw string literal for embedding in generated code.
    Similar to rust_raw_string but handles additional edge cases.
    """
    return rust_raw_string(pattern)



# ---------------------------------------------------------------------------
# Lexer introspection
# ---------------------------------------------------------------------------

def get_lexer_class(lexer_name: str):
    """
    Import a pygments lexer class by name.
    Returns the class or None if not found.
    """
    try:
        from pygments.lexers._mapping import LEXERS
    except ImportError:
        return None

    if lexer_name not in LEXERS:
        return None

    module_path, class_name = LEXERS[lexer_name][0], lexer_name
    try:
        module = importlib.import_module(module_path)
        return getattr(module, class_name, None)
    except Exception:
        return None


def extract_lexer_info(cls) -> dict:
    """
    Extract lexer metadata from a pygments lexer class.
    Returns dict with: name, aliases, filenames, mimetypes, priority, analyse_text
    """
    info = {
        'name': getattr(cls, 'name', cls.__name__.replace('Lexer', '')),
        'aliases': list(getattr(cls, 'aliases', [])),
        'filenames': list(getattr(cls, 'filenames', [])),
        'mimetypes': list(getattr(cls, 'mimetypes', [])),
        'priority': getattr(cls, 'priority', 0),
        'analyse_text': None,
    }

    # Extract analyse_text if present
    if hasattr(cls, 'analyse_text') and callable(getattr(cls, 'analyse_text')):
        # Get the source code of analyse_text
        try:
            source = inspect.getsource(cls.analyse_text)
            info['analyse_text'] = source
        except Exception:
            pass

    return info


def extract_token_definitions(cls) -> dict:
    """
    Extract and process token definitions from a lexer class.

    Returns a dict of {state_name: [(regex, token, new_state), ...]}
    Handles inheritance via get_tokendefs().
    """
    # Use the metaclass's get_tokendefs to merge inheritance
    try:
        tokendefs = cls.get_tokendefs()
    except Exception:
        return {}

    result = {}
    for state, rules in tokendefs.items():
        processed_rules = []
        for rule in rules:
            processed = process_rule(rule, cls)
            if processed:
                processed_rules.append(processed)
        result[state] = processed_rules

    return result


def process_rule(rule, cls) -> Optional[tuple]:
    """
    Process a single token definition rule.
    Returns (regex_str, token_str, new_state) or None if unhandled.
    """
    if not isinstance(rule, tuple) or len(rule) < 2:
        return None

    regex_part, token_part = rule[0], rule[1]

    # Handle 'include' directive — skip (not supported in generated code)
    if type(regex_part).__name__ == 'include':
        return None

    # Handle 'inherit' — skip (handled by inheritance)
    if type(token_part).__name__ == '_inherit' or str(token_part) == 'inherit':
        return None

    # Handle 'default' directive — skip
    if hasattr(token_part, 'state'):
        return None

    # Handle 'combined' — skip (not supported in generated code)
    if type(token_part).__name__ == 'combined':
        return None

    # Handle 'words' (Future) — expand to regex
    if hasattr(regex_part, 'get') and callable(getattr(regex_part, 'get')):
        try:
            expanded = regex_part.get()
            regex_str = expanded
        except Exception:
            return None
    elif callable(regex_part):
        # It's a compiled regex match function — extract pattern
        if hasattr(regex_part, '__self__'):
            # It's a bound method from the metaclass
            try:
                regex_str = regex_part.__self__.pattern
            except Exception:
                return None
        else:
            # It's a callback function (e.g., bygroups)
            return None
    else:
        regex_str = regex_part

    # Convert token type
    rust_token = python_token_to_rust(token_part)

    # Handle new_state
    new_state = None
    if len(rule) > 2:
        ns = rule[2]
        if isinstance(ns, str):
            if ns == '#pop':
                new_state = '#pop'
            elif ns == '#push':
                new_state = '#push'
            else:
                new_state = ns
        elif isinstance(ns, tuple):
            new_state = ','.join(ns)
        elif callable(ns):
            # It's a callback (bygroups) — skip
            return None

    return (regex_str, rust_token, new_state)


# ---------------------------------------------------------------------------
# Rust code generation
# ---------------------------------------------------------------------------

def rust_escape(s: str) -> str:
    """Escape a string for Rust string literals."""
    s = s.replace('\\', '\\\\')
    s = s.replace('"', '\\"')
    s = s.replace('\n', '\\n')
    s = s.replace('\r', '\\r')
    s = s.replace('\t', '\\t')
    return s


def rust_token_for_state(state_name: str) -> str:
    """Generate a Rust Token constant for a state-like token reference."""
    return "TEXT"  # fallback


def generate_lexer_rust(name: str, info: dict, tokendefs: dict) -> str:
    """
    Generate Rust source code for a lexer.
    """
    lines: list[str] = []

    # Class name
    class_name = f"{name}Lexer" if not name.endswith('Lexer') else name

    # Header
    lines.append(f"// AUTO-GENERATED by generators/gen_lexers.py — DO NOT EDIT BY HAND")
    lines.append(f"// Lexer: {name}")
    lines.append("")
    lines.append("use crate::token::Token;")
    lines.append("use crate::lexer::{Lexer, RegexLexer, LexerRule, LexerAction};")
    lines.append("use crate::scanner::TokenPattern;")
    lines.append("")
    lines.append(f"/// Auto-generated {name} lexer.")
    lines.append(f"pub struct {class_name} {{")
    lines.append("    inner: RegexLexer,")
    lines.append("}")
    lines.append("")
    lines.append(f"impl {class_name} {{")
    lines.append("    pub fn new() -> Self {")
    lines.append(f'        let mut inner = RegexLexer::new("{info["name"]}");')

    # Aliases
    if info['aliases']:
        lines.append("        inner.aliases = vec![")
        for alias in info['aliases']:
            lines.append(f'            "{alias}",')
        lines.append("        ];")

    # Filenames
    if info['filenames']:
        lines.append("        inner.filenames = vec![")
        for fname in info['filenames']:
            lines.append(f'            "{rust_escape(fname)}",')
        lines.append("        ];")

    # Mimetypes
    if info['mimetypes']:
        lines.append("        inner.mimetypes = vec![")
        for mt in info['mimetypes']:
            lines.append(f'            "{rust_escape(mt)}",')
        lines.append("        ];")

    lines.append("")

    # Generate states
    state_count = 0
    for state_name, rules in tokendefs.items():
        if not rules:
            continue

        # Skip 'root' if it's empty (just inheritance placeholder)
        if state_name == 'root' and len(rules) == 0:
            continue

        state_count += 1

        if state_name == 'root':
            lines.append("        // Root state")
            lines.append("        let mut root_rules: Vec<LexerRule> = Vec::new();")
            lines.append("")
        else:
            lines.append(f"        // State: {state_name}")
            lines.append(f"        inner.states.insert(\"{state_name}\".to_string(), vec![")

        for regex_str, rust_token, new_state in rules:
            if not regex_str or not rust_token:
                continue

            # Convert regex
            rust_regex = convert_regex(regex_str)

            # Generate Rust raw string literal for the pattern
            rust_pattern = rust_raw_string_for_code(rust_regex)

            # Generate rule
            if new_state and new_state.startswith('#pop'):
                # Pop action
                try:
                    pop_n = int(new_state[4:]) if new_state[4:].isdigit() else 1
                except ValueError:
                    pop_n = 1
                rule_code = f'LexerRule {{ pattern: TokenPattern::new({rust_pattern}, Token::{rust_token}).unwrap(), action: LexerAction::PopN({pop_n}) }}'

            elif new_state and new_state == '#push':
                rule_code = f'LexerRule {{ pattern: TokenPattern::new({rust_pattern}, Token::{rust_token}).unwrap(), action: LexerAction::Push("root".to_string()) }}'

            elif new_state and ',' in new_state:
                # Multiple state push
                states = [s for s in new_state.split(',') if s not in ('#pop', '#push')]
                if states:
                    rule_code = f'LexerRule {{ pattern: TokenPattern::new({rust_pattern}, Token::{rust_token}).unwrap(), action: LexerAction::Push("{states[0]}".to_string()) }}'
                else:
                    continue

            elif new_state:
                rule_code = f'LexerRule {{ pattern: TokenPattern::new({rust_pattern}, Token::{rust_token}).unwrap(), action: LexerAction::Push("{new_state}".to_string()) }}'

            else:
                rule_code = f'LexerRule {{ pattern: TokenPattern::new({rust_pattern}, Token::{rust_token}).unwrap(), action: LexerAction::token(Token::{rust_token}) }}'

            # Emit rule in correct context
            if state_name == 'root':
                lines.append(f'            root_rules.push({rule_code});')
            else:
                lines.append(f'            {rule_code},')

        if state_name == 'root':
            lines.append("")
            lines.append("        inner.states.insert(\"root\".to_string(), root_rules);")
        else:
            lines.append("        ]);")

        lines.append("")

    if state_count == 0:
        # No states generated — add a placeholder
        lines.append("        // No token rules found")
        lines.append("        inner.states.insert(\"root\".to_string(), vec![]);")
        lines.append("")

    lines.append(f"        {class_name} {{ inner }}")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    # Lexer trait impl
    lines.append("impl Lexer for " + class_name + " {")
    lines.append("    fn get_tokens(&self, code: &str) -> Vec<(Token, String)> {")
    lines.append("        self.inner.get_tokens(code)")
    lines.append("    }")
    lines.append("")
    lines.append("    fn name(&self) -> &str {")
    lines.append("        &self.inner.name")
    lines.append("    }")
    lines.append("")
    lines.append("    fn aliases(&self) -> &[&str] {")
    lines.append("        &self.inner.aliases")
    lines.append("    }")
    lines.append("")
    lines.append("    fn filenames(&self) -> &[&str] {")
    lines.append("        &self.inner.filenames")
    lines.append("    }")
    lines.append("")
    lines.append("    fn mimetypes(&self) -> &[&str] {")
    lines.append("        &self.inner.mimetypes")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    return '\n'.join(lines)


# ---------------------------------------------------------------------------
# Registry update
# ---------------------------------------------------------------------------

def generate_registry_entry(name: str, info: dict) -> str:
    """Generate a registry entry for a lexer."""
    class_name = f"{name}Lexer" if not name.endswith('Lexer') else name
    aliases = info['aliases']
    filenames = info['filenames']
    mimetypes = info['mimetypes']
    priority = info['priority']

    lines = []
    lines.append(f"    // {name} ({class_name})")
    lines.append(f'    registry.register_lexer("{info["name"]}", "{class_name}::new");')
    for alias in aliases:
        lines.append(f'    registry.register_lexer_alias("{alias}", "{info["name"]}");')
    for fname in filenames:
        lines.append(f'    registry.register_filename("{fname}", "{info["name"]}");')
    for mt in mimetypes:
        lines.append(f'    registry.register_mimetype("{mt}", "{info["name"]}");')
    if priority:
        lines.append(f'    registry.set_priority("{info["name"]}", {priority});')
    lines.append("")

    return '\n'.join(lines)


# ---------------------------------------------------------------------------
# Module mod.rs update
# ---------------------------------------------------------------------------

def update_mod_rs(class_name: str) -> str:
    """Generate the mod.rs module declaration for a new lexer."""
    # Convert class name to file name: Python3Lexer -> python3.rs
    # But we already have python.rs, so handle special cases
    name_map = {
        'Python3Lexer': 'python',
        'PythonLexer': 'python',
        'CLexer': 'cpp',
        'CppLexer': 'cpp',
        'HtmlLexer': 'htmlxml',
        'XmlLexer': 'htmlxml',
    }

    file_name = name_map.get(class_name, class_name.replace('Lexer', '').lower())
    return f"pub mod {file_name};"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def generate_all(output_dir: Optional[Path] = None) -> dict:
    """
    Generate all supported lexers.
    Returns dict of {lexer_name: status}.
    """
    if output_dir is None:
        output_dir = LEXER_OUT_DIR

    from pygments.lexers._mapping import LEXERS

    results = {}

    # Filter out template/Delegating lexers (require ExtendedRegexLexer)
    skip_prefixes = ('Template', 'Delegating', 'Angular', 'Cheetah',
                     'Coldfusion', 'Cython', 'Genshi', 'GenshiPython',
                     'GenshiPytb', 'GenshiPytbout', 'GenshiPytraceback',
                     'GenshiTb', 'GenshiTbout', 'GenshiTraceback',
                     'Mako', 'Myghty', 'NewstyleCheetah', 'Pylons',
                     'Web2Cheetah', 'WebCheetah')

    # Process each lexer
    for lexer_name in sorted(LEXERS.keys()):
        module_path, class_name = LEXERS[lexer_name][0], lexer_name

        # Skip template/delegating lexers
        if any(module_path.startswith(f'pygments.lexers.{p}') for p in
               ['templates', 'markup']):
            results[lexer_name] = 'skipped_template'
            continue

        # Skip if already manually ported (check for existing file)
        name_map = {
            'Python3Lexer': 'python',
            'PythonLexer': 'python',
            'CLexer': 'cpp',
            'CppLexer': 'cpp',
            'HtmlLexer': 'htmlxml',
            'XmlLexer': 'htmlxml',
        }
        file_name = name_map.get(class_name, class_name.replace('Lexer', '').lower())
        existing_file = output_dir / f"{file_name}.rs"
        if existing_file.exists():
            results[lexer_name] = 'already_exists'
            continue

        # Import and inspect
        try:
            module = importlib.import_module(module_path)
            cls = getattr(module, class_name, None)
            if cls is None:
                results[lexer_name] = 'import_error'
                continue
        except Exception as e:
            results[lexer_name] = f'import_error: {e}'
            continue

        # Extract info
        try:
            info = extract_lexer_info(cls)
            tokendefs = extract_token_definitions(cls)
        except Exception as e:
            results[lexer_name] = f'extract_error: {e}'
            continue

        # Skip if no token definitions
        if not any(rules for rules in tokendefs.values()):
            results[lexer_name] = 'no_tokens'
            continue

        # Generate Rust code
        try:
            rust_code = generate_lexer_rust(class_name, info, tokendefs)
            output_file = output_dir / f"{file_name}.rs"
            # Handle surrogate characters by replacing them
            rust_code = ''.join(ch for ch in rust_code if not (0xD800 <= ord(ch) <= 0xDFFF))
            output_file.write_text(rust_code, encoding='utf-8')
            results[lexer_name] = f'generated:{output_file}'
        except Exception as e:
            results[lexer_name] = f'generate_error: {e}'

    return results


def generate_single(lexer_name: str, output_dir: Optional[Path] = None) -> dict:
    """Generate a single lexer."""
    if output_dir is None:
        output_dir = LEXER_OUT_DIR

    from pygments.lexers._mapping import LEXERS

    if lexer_name not in LEXERS:
        print(f"Error: Unknown lexer '{lexer_name}'")
        print(f"Available lexers: {', '.join(sorted(LEXERS.keys())[:20])}... ({len(LEXERS)} total)")
        return {}

    module_path, class_name = LEXERS[lexer_name][0], lexer_name

    try:
        module = importlib.import_module(module_path)
        cls = getattr(module, class_name, None)
        if cls is None:
            print(f"Error: Could not find class {class_name} in {module_path}")
            return {}
    except Exception as e:
        print(f"Error importing {module_path}: {e}")
        return {}

    info = extract_lexer_info(cls)
    tokendefs = extract_token_definitions(cls)

    if not any(rules for rules in tokendefs.values()):
        print(f"No token definitions found for {lexer_name}")
        return {}

    name_map = {
        'Python3Lexer': 'python',
        'PythonLexer': 'python',
        'CLexer': 'cpp',
        'CppLexer': 'cpp',
        'HtmlLexer': 'htmlxml',
        'XmlLexer': 'htmlxml',
    }
    file_name = name_map.get(class_name, class_name.replace('Lexer', '').lower())
    output_file = output_dir / f"{file_name}.rs"

    rust_code = generate_lexer_rust(class_name, info, tokendefs)
    output_file.write_text(rust_code, encoding='utf-8')

    print(f"Generated {output_file}")
    print(f"  Lexer: {info['name']}")
    print(f"  Aliases: {info['aliases']}")
    print(f"  States: {len(tokendefs)}")
    total_rules = sum(len(rules) for rules in tokendefs.values())
    print(f"  Total rules: {total_rules}")

    return {lexer_name: f'generated:{output_file}'}


def main():
    parser = argparse.ArgumentParser(
        description='Generate Rust lexer implementations from pygments source.',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python generators/gen_lexers.py                    # Generate all supported lexers
  python generators/gen_lexers.py --lexer python     # Generate only Python lexer
  python generators/gen_lexers.py --all              # Generate all (same as default)
  python generators/gen_lexers.py --output /tmp/out  # Output to different directory
        """
    )
    parser.add_argument('--output', '-o', type=Path, default=None,
                        help='Output directory (default: carthamin-core/src/lexer/)')
    parser.add_argument('--lexer', '-l', type=str, default=None,
                        help='Generate a specific lexer by name')
    parser.add_argument('--all', '-a', action='store_true',
                        help='Generate all supported lexers (default)')

    args = parser.parse_args()

    output_dir = args.output or LEXER_OUT_DIR
    output_dir.mkdir(parents=True, exist_ok=True)

    if args.lexer:
        results = generate_single(args.lexer, output_dir)
    else:
        results = generate_all(output_dir)

    # Print summary
    print(f"\n{'='*60}")
    print(f"Generation complete")
    print(f"{'='*60}")

    generated = [k for k, v in results.items() if v.startswith('generated:')]
    skipped = [k for k, v in results.items() if v == 'skipped_template']
    existing = [k for k, v in results.items() if v == 'already_exists']
    errors = [k for k, v in results.items() if v.startswith('error') or v.startswith('generate_error') or v.startswith('extract_error') or v.startswith('import_error')]
    no_tokens = [k for k, v in results.items() if v == 'no_tokens']

    print(f"  Generated: {len(generated)}")
    print(f"  Skipped (template): {len(skipped)}")
    print(f"  Already exists: {len(existing)}")
    print(f"  No tokens: {len(no_tokens)}")
    print(f"  Errors: {len(errors)}")

    if generated:
        print(f"\nGenerated lexers:")
        for name in generated[:10]:
            print(f"  - {name}")
        if len(generated) > 10:
            print(f"  ... and {len(generated)-10} more")

    if errors:
        print(f"\nErrors:")
        for name in errors[:10]:
            print(f"  - {name}: {results[name]}")
        if len(errors) > 10:
            print(f"  ... and {len(errors)-10} more")


if __name__ == '__main__':
    main()
