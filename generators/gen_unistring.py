#!/usr/bin/env python3
"""Generate Rust unistring module from pygments.unistring source file.

Reads pygments/unistring.py and extracts category string assignments,
converting Python escape sequences to Rust \\u{XXXX} format.
Outputs carthamin-core/src/unistring.rs with static &str constants.
"""
import re
import sys
import os


# Categories to export (all 30 Unicode categories + xid variants)
CATEGORIES = [
    'Cc', 'Cf', 'Cn', 'Co', 'Cs',
    'Ll', 'Lm', 'Lo', 'Lt', 'Lu',
    'Mc', 'Me', 'Mn',
    'Nd', 'Nl', 'No',
    'Pc', 'Pd', 'Pe', 'Pf', 'Pi', 'Po', 'Ps',
    'Sc', 'Sk', 'Sm', 'So',
    'Zl', 'Zp', 'Zs',
    'xid_start', 'xid_continue',
]


def find_unistring_py() -> str:
    """Find pygments/unistring.py relative to this script or cwd."""
    # Try relative to script location
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_dir = os.path.dirname(script_dir)
    candidate = os.path.join(project_dir, 'pygments', 'unistring.py')
    if os.path.exists(candidate):
        return candidate
    # Try relative to cwd
    candidate = os.path.join('pygments', 'unistring.py')
    if os.path.exists(candidate):
        return candidate
    raise FileNotFoundError(
        f"Cannot find pygments/unistring.py. Searched: {candidate}"
    )


def extract_categories(source_path: str) -> dict:
    """Extract NAME = '...' or NAME = "..." assignments from unistring.py source."""
    categories = {}
    with open(source_path, 'r', encoding='utf-8') as f:
        content = f.read()

    for line in content.splitlines():
        stripped = line.strip()
        # Match: NAME = '...'  or  NAME = "..."
        m = re.match(r'^([A-Za-z_]\w+)\s*=\s*([\"\'])', stripped)
        if not m:
            continue
        name = m.group(1)
        quote_char = m.group(2)
        rest = stripped[m.end():]

        # Parse string content handling escape sequences
        value = []
        i = 0
        while i < len(rest):
            if rest[i] == '\\' and i + 1 < len(rest):
                # Escape sequence: keep both chars
                value.append(rest[i])
                value.append(rest[i + 1])
                i += 2
                continue
            if rest[i] == quote_char:
                # Closing quote
                break
            value.append(rest[i])
            i += 1

        if name in CATEGORIES or name in ('cats',):
            categories[name] = ''.join(value)

    return categories


def convert_escapes(s: str) -> str:
    """Convert Python escape sequences in the source string to Rust \\u{XXXX} format.

    Input is the raw string content from the Python source file (between quotes).
    Python escapes like \\x00, \\uXXXX, \\UXXXXXXXX are converted to Rust format.
    Literal characters (a-z, A-Z, 0-9, -, etc.) pass through unchanged.
    """
    result = []
    i = 0
    while i < len(s):
        c = s[i]
        if c == '\\' and i + 1 < len(s):
            next_c = s[i + 1]
            if next_c == 'x' and i + 3 < len(s):
                # \xNN -> \u{00NN}
                code = s[i+2:i+4]
                result.append(f'\\u{{00{code}}}')
                i += 4
                continue
            elif next_c == 'u' and i + 5 < len(s):
                # \uXXXX -> \u{XXXX}
                code = s[i+2:i+6]
                result.append(f'\\u{{{code}}}')
                i += 6
                continue
            elif next_c == 'U' and i + 9 < len(s):
                # \UXXXXXXXX -> \u{XXXXX} (strip leading zeros, max 6 hex digits)
                code = s[i+2:i+10]
                # Rust \u{} only supports up to 6 hex digits; strip leading zeros
                code_int = int(code, 16)
                result.append(f'\\u{{{code_int:x}}}')
                i += 10
                continue
            elif next_c == '\\':
                # \\\\ -> literal backslash, output as \\\\ in Rust string
                result.append('\\\\\\\\')
                i += 2
                continue
            elif next_c == "'":
                # \\' -> literal single quote
                result.append("'")
                i += 2
                continue
            elif next_c == '"':
                # \\" -> literal double quote (needs escaping in Rust &str)
                result.append('\\"')
                i += 2
                continue
            # Unknown escape — pass through as-is
            result.append(c)
            result.append(next_c)
            i += 2
            continue
        elif c == '"':
            # Double quote needs escaping in Rust &str
            result.append('\\"')
            i += 1
            continue
        elif c == '\\':
            # Lone backslash at end — escape it
            result.append('\\\\')
            i += 1
            continue
        # Regular character — pass through
        result.append(c)
        i += 1
    return ''.join(result)


def escape_for_rust_str(s: str) -> str:
    """Additional escaping for characters that need it in Rust &str.

    After convert_escapes, we still need to escape:
    - Tab, newline, etc. (if any literal control chars slipped through)
    """
    result = []
    for c in s:
        o = ord(c)
        if c == '\n':
            result.append('\\n')
        elif c == '\r':
            result.append('\\r')
        elif c == '\t':
            result.append('\\t')
        elif o < 0x20:
            result.append(f'\\x{{o:02x}}')
        else:
            result.append(c)
    return ''.join(result)


def main():
    source_path = find_unistring_py()
    out_path = sys.argv[1] if len(sys.argv) > 1 else 'carthamin-core/src/unistring.rs'

    categories = extract_categories(source_path)

    lines = [
        '// Auto-generated by generators/gen_unistring.py',
        '// DO NOT EDIT MANUALLY',
        '//',
        '// Contains Unicode character category data for regex-based lexers.',
        '// Generated from pygments.unistring (Unicode 11.0.0).',
        '',
    ]

    # Track which categories we actually export
    exported = []

    # Generate static constants for each category
    for cat in CATEGORIES:
        if cat not in categories:
            print(f"  Warning: {cat} not found in unistring.py", file=sys.stderr)
            continue

        raw_value = categories[cat]
        # CS (surrogates) are invalid in Rust string literals (not valid Unicode scalar values)
        # They are a UTF-16 concept that doesn't apply to UTF-8 text
        if cat == 'Cs':
            rust_value = ''
        else:
            rust_value = convert_escapes(raw_value)
            rust_value = escape_for_rust_str(rust_value)
        rust_name = cat.upper()

        lines.append(f'/// Unicode category: {cat}')
        lines.append(f'pub static {rust_name}: &str = "{rust_value}";')
        lines.append('')
        exported.append(cat)

    # Generate combine() function
    lines.extend([
        '/// Combine multiple Unicode category strings into one character class.',
        '/// Equivalent to pygments.unistring.combine(*cats).',
        'pub fn combine(cats: &[&str]) -> String {',
        '    let mut result = String::new();',
        '    for cat in cats {',
        '        let value = match *cat {',
    ])

    for cat in exported:
        rust_name = cat.upper()
        lines.append(f'            "{cat}" => {rust_name},')

    lines.extend([
        '            _ => "",',
        '        };',
        '        result.push_str(value);',
        '    }',
        '    result',
        '}',
        '',
        '/// Get all Unicode categories except the ones specified.',
        '/// Equivalent to pygments.unistring.allexcept(*cats).',
        'pub fn allexcept(exclude: &[&str]) -> String {',
        '    let all_cats: &[&str] = &[',
    ])

    category_names = [c for c in exported if c not in ('xid_start', 'xid_continue')]
    lines.append('        ' + ', '.join(f'"{c}"' for c in category_names) + ',')
    lines.extend([
        '    ];',
        '    let exclude_set: std::collections::HashSet<_> = exclude.iter().collect();',
        '    let mut result = String::new();',
        '    for cat in all_cats {',
        '        if !exclude_set.contains(cat) {',
        '            let value = match *cat {',
    ])

    for cat in category_names:
        rust_name = cat.upper()
        lines.append(f'                "{cat}" => {rust_name},')

    lines.extend([
        '                _ => "",',
        '            };',
        '            result.push_str(value);',
        '        }',
        '    }',
        '    result',
        '}',
    ])

    # Generate unit tests
    lines.extend([
        '',
        '#[cfg(test)]',
        'mod tests {',
        '    use super::*;',
        '',
        '    #[test]',
        '    fn test_xid_start_non_empty() {',
        '        assert!(!XID_START.is_empty(), "XID_START should not be empty");',
        '    }',
        '',
        '    #[test]',
        '    fn test_xid_continue_non_empty() {',
        '        assert!(!XID_CONTINUE.is_empty(), "XID_CONTINUE should not be empty");',
        '    }',
        '',
        '    #[test]',
        '    fn test_xid_start_contains_ascii_ranges() {',
        '        assert!(XID_START.contains("A-Z"), "XID_START should contain A-Z range");',
        '        assert!(XID_START.contains("a-z"), "XID_START should contain a-z range");',
        '        assert!(XID_START.contains("_"), "XID_START should contain underscore");',
        '    }',
        '',
        '    #[test]',
        '    fn test_xid_start_contains_unicode() {',
        '        // Chinese characters (CJK Unified Ideographs) should be in xid_start',
        '        // U+4E00 is the start of CJK Unified Ideographs',
        '        assert!(XID_START.contains("\\u{4e00}"), "XID_START should contain CJK characters");',
        '    }',
        '',
        '    #[test]',
        '    fn test_xid_continue_contains_digits() {',
        '        assert!(XID_CONTINUE.contains("0-9"), "XID_CONTINUE should contain 0-9 range");',
        '    }',
        '',
        '    #[test]',
        '    fn test_combine_basic() {',
        '        let combined = combine(&["Lu", "Ll"]);',
        '        assert!(!combined.is_empty(), "Combined string should not be empty");',
        '        assert!(combined.contains("A-Z"), "Combined Lu+Ll should contain A-Z");',
        '        assert!(combined.contains("a-z"), "Combined Lu+Ll should contain a-z");',
        '    }',
        '',
        '    #[test]',
        '    fn test_allexcept_basic() {',
        '        let result = allexcept(&["Nd"]);',
        '        assert!(!result.is_empty(), "allexcept result should not be empty");',
        '        // Should NOT contain digits since we excluded Nd',
        '        assert!(!result.contains("0-9"), "allexcept(&[Nd]) should not contain digits");',
        '    }',
        '',
        '    #[test]',
        '    fn test_cc_contains_control_chars() {',
        '        // Cc (control characters) should contain \\u{0000} range',
        '        assert!(CC.contains("\\u{0000}"), "CC should contain null character range");',
        '    }',
        '',
        '    #[test]',
        '    fn test_pattern_compiles_for_python_ident() {',
        '        use regex::Regex;',
        '        let pattern = format!("[{}][{}]*", XID_START, XID_CONTINUE);',
        '        let re = Regex::new(&pattern);',
        '        assert!(re.is_ok(), "Python identifier pattern should compile: {}", pattern.chars().take(80).collect::<String>());',
        '    }',
        '',
        '}',
    ])

    with open(out_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines) + '\n')

    print(f"Generated {out_path}")
    print(f"  Categories exported: {len(exported)}")
    print(f"  XID variants: xid_start, xid_continue")
    print(f"  Source: {source_path}")


if __name__ == '__main__':
    main()
