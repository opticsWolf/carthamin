use std::collections::HashMap;

/// Escape special regex characters in a string.
pub fn escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 10);
    for c in s.chars() {
        match c {
            '.' | '^' | '$' | '\\' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '#' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Find the longest common prefix among strings.
pub fn common_prefix(strings: &[&str]) -> (String, String) {
    if strings.is_empty() {
        return (String::new(), String::new());
    }
    let first = strings[0];
    let last = strings[strings.len() - 1];

    let mut prefix_len = 0;
    let first_chars: Vec<char> = first.chars().collect();
    let last_chars: Vec<char> = last.chars().collect();
    let min_len = first_chars.len().min(last_chars.len());

    for i in 0..min_len {
        if first_chars[i] == last_chars[i] {
            prefix_len += 1;
        } else {
            break;
        }
    }

    let prefix: String = first.chars().take(prefix_len).collect();
    let rest = if prefix_len > 0 {
        first.chars().skip(prefix_len).collect()
    } else {
        String::new()
    };

    (prefix, rest)
}

/// Find the longest common suffix among strings.
pub fn common_suffix(strings: &[&str]) -> (String, String) {
    if strings.is_empty() {
        return (String::new(), String::new());
    }
    let first = strings[0];
    let last = strings[strings.len() - 1];

    let first_chars: Vec<char> = first.chars().collect();
    let last_chars: Vec<char> = last.chars().collect();

    let mut suffix_len = 0;
    let min_len = first_chars.len().min(last_chars.len());

    for i in 0..min_len {
        if first_chars[first_chars.len() - 1 - i] == last_chars[last_chars.len() - 1 - i] {
            suffix_len += 1;
        } else {
            break;
        }
    }

    let suffix: String = last_chars.iter().skip(last_chars.len() - suffix_len).collect();
    let rest = if suffix_len > 0 {
        first_chars.iter().take(first_chars.len() - suffix_len).collect()
    } else {
        String::new()
    };

    (suffix, rest)
}

/// Build a character class from a set of letters.
pub fn make_charset(letters: &[char]) -> String {
    let mut result = String::from("[");
    for &c in letters {
        match c {
            '-' | ']' | '\\' | '[' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result.push(']');
    result
}

/// Generate an optimized regex that matches any string in the sorted list.
/// This is the core of `regex_opt` from Pygments.
pub fn regex_opt_inner(strings: &[&str], open_paren: char) -> String {
    if strings.is_empty() {
        return String::new();
    }

    if strings.len() == 1 {
        return escape(strings[0]);
    }

    // Check for common prefix
    let (prefix, _) = common_prefix(strings);
    if !prefix.is_empty() {
        let rest: Vec<String> = strings.iter().map(|s| {
            let chars: Vec<char> = s.chars().collect();
            let prefix_chars: Vec<char> = prefix.chars().collect();
            chars.into_iter().skip(prefix_chars.len()).collect()
        }).collect();

        // Group by first character of remainder
        let mut groups: HashMap<Option<char>, Vec<String>> = HashMap::new();
        for s in &rest {
            let first = s.chars().next();
            groups.entry(first).or_default().push(s.clone());
        }

        let mut alternatives: Vec<String> = Vec::new();
        for (_, group) in groups.iter() {
            if group.len() == 1 {
                alternatives.push(escape(&group[0]));
            } else {
                let group_strs: Vec<&str> = group.iter().map(|s| s.as_str()).collect();
                alternatives.push(regex_opt_inner(&group_strs, open_paren));
            }
        }

        let _join_char = if alternatives.len() == 1 {
            String::new()
        } else {
            format!("{}(|", open_paren)
        };

        if alternatives.len() == 1 {
            format!("{}{}", escape(&prefix), alternatives[0])
        } else {
            format!("{}{}{})", escape(&prefix), alternatives.join("|"), close_paren(open_paren))
        }
    } else {
        // Check for single-character grouping
        let first_chars: Vec<char> = strings.iter().map(|s| s.chars().next().unwrap_or('\0')).collect();
        let unique_first: Vec<char> = {
            let mut seen = std::collections::HashSet::new();
            first_chars.into_iter().filter(|c| seen.insert(*c)).collect()
        };

        if unique_first.len() < strings.len() / 2 {
            // Group by first character
            let mut groups: HashMap<char, Vec<String>> = HashMap::new();
            for s in strings {
                let chars: Vec<char> = s.chars().collect();
                let first = chars[0];
                let rest: String = chars.into_iter().skip(1).collect();
                groups.entry(first).or_default().push(rest);
            }

            let mut alternatives: Vec<String> = Vec::new();
            for (ch, group) in &groups {
                if group.len() == 1 {
                    if group[0].is_empty() {
                        alternatives.push(escape(&ch.to_string()));
                    } else {
                        alternatives.push(format!("{}{}", escape(&ch.to_string()), escape(&group[0])));
                    }
                } else {
                    let group_strs: Vec<&str> = group.iter().map(|s| s.as_str()).collect();
                    let inner = regex_opt_inner(&group_strs, open_paren);
                    alternatives.push(format!("{}{}", escape(&ch.to_string()), inner));
                }
            }

            if alternatives.len() == 1 {
                alternatives[0].clone()
            } else {
                format!("{}(|{}{})", open_paren, alternatives.join("|"), close_paren(open_paren))
            }
        } else {
            // Simple alternation
            let escaped: Vec<String> = strings.iter().map(|s| escape(s)).collect();
            format!("{}(|{}{})", open_paren, escaped.join("|"), close_paren(open_paren))
        }
    }
}

fn close_paren(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => ')',
    }
}

/// Main entry point: generate optimized regex for a list of strings.
pub fn regex_opt(strings: &[&str], prefix: &str, suffix: &str) -> String {
    let sorted: Vec<&str> = {
        let mut s: Vec<&str> = strings.to_vec();
        s.sort();
        s
    };
    let inner = regex_opt_inner(&sorted, '(');
    format!("{}{}{}", escape(prefix), inner, escape(suffix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape("hello.world"), "hello\\.world");
        assert_eq!(escape("test"), "test");
    }

    #[test]
    fn test_common_prefix() {
        let strings = vec!["hello", "help", "her"];
        let (prefix, _) = common_prefix(&strings);
        assert_eq!(prefix, "he");
    }

    #[test]
    fn test_regex_opt_basic() {
        let result = regex_opt(&["if", "else", "while"], "", "");
        assert!(!result.is_empty());
        // Should compile as valid regex
        let _re = regex::Regex::new(&result).expect("Generated regex should be valid");
    }

    #[test]
    fn test_regex_opt_with_prefix() {
        let result = regex_opt(&["foo", "bar"], "\\b", "\\b");
        assert!(result.starts_with("\\\\b"));
    }
}
