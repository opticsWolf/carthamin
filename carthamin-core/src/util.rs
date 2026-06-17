use std::collections::HashSet;
use regex::Regex;

/// Error types for option parsing.
#[derive(Debug, Clone)]
pub enum OptionError {
    InvalidValue(String),
    MissingOption(String),
}

impl std::fmt::Display for OptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionError::InvalidValue(msg) => write!(f, "Invalid option value: {}", msg),
            OptionError::MissingOption(msg) => write!(f, "Missing option: {}", msg),
        }
    }
}

impl std::error::Error for OptionError {}

/// Get a boolean option from the options map.
pub fn get_bool_opt(options: &std::collections::HashMap<String, String>, optname: &str, default: bool) -> Result<bool, OptionError> {
    match options.get(optname) {
        None => Ok(default),
        Some(val) => {
            let lower = val.to_lowercase();
            match lower.as_str() {
                "1" | "true" | "t" | "yes" | "y" => Ok(true),
                "0" | "false" | "f" | "no" | "n" => Ok(false),
                _ => Err(OptionError::InvalidValue(format!(
                    "Value for option {} must be a boolean", optname
                ))),
            }
        }
    }
}

/// Get an integer option from the options map.
pub fn get_int_opt(options: &std::collections::HashMap<String, String>, optname: &str, default: i64) -> Result<i64, OptionError> {
    match options.get(optname) {
        None => Ok(default),
        Some(val) => val.parse::<i64>().map_err(|_| {
            OptionError::InvalidValue(format!("Value for option {} must be an integer", optname))
        }),
    }
}

/// Get a list option (comma or space separated) from the options map.
pub fn get_list_opt(options: &std::collections::HashMap<String, String>, optname: &str, default: Vec<String>) -> Result<Vec<String>, OptionError> {
    match options.get(optname) {
        None => Ok(default),
        Some(val) => {
            let list: Vec<String> = val.split([',', ' '])
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            Ok(list)
        }
    }
}

/// Get a choice option (must be one of the allowed values).
pub fn get_choice_opt(options: &std::collections::HashMap<String, String>, optname: &str, allowed: &[&str], default: &str) -> Result<String, OptionError> {
    match options.get(optname) {
        None => Ok(default.to_string()),
        Some(val) => {
            let lower = val.to_lowercase();
            if allowed.iter().any(|a| a.to_lowercase() == lower) {
                Ok(val.to_string())
            } else {
                Err(OptionError::InvalidValue(format!(
                    "Value for option {} must be one of {}", optname, allowed.join(", ")
                )))
            }
        }
    }
}

/// Check if a shebang line matches a regex pattern.
pub fn shebang_matches(text: &str, pattern: &str) -> bool {
    if let Some(first_line) = text.lines().next() {
        if !first_line.starts_with("#!") {
            return false;
        }
        // Extract the executable path from shebang
        let shebang = &first_line[2..].trim_start();
        // Handle "#!/usr/bin/env python" style
        let search_str = if shebang.starts_with("env ") {
            &shebang[4..]
        } else if shebang.starts_with("usr/bin/env ") {
            &shebang[10..]
        } else {
            // Get last component of path
            shebang.split('/').last().unwrap_or(shebang)
        };
        match regex::Regex::new(pattern) {
            Ok(re) => re.is_match(search_str),
            Err(_) => false,
        }
    } else {
        false
    }
}

/// Check if text looks like XML (has XML declaration or tags).
pub fn looks_like_xml(text: &str) -> bool {
    static XML_DECL_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?s)<\?xml[^>]+>").unwrap()
    });
    static TAG_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?s)<[^<>]+>").unwrap()
    });
    static DOCTYPE_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?s)<!DOCTYPE[^>]*>").unwrap()
    });

    XML_DECL_RE.is_match(text) || DOCTYPE_RE.is_match(text) || TAG_RE.is_match(text)
}

/// HTML-escape a string.
pub fn html_escape(s: &str, quote: bool) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' if quote => result.push_str("&quot;"),
            c => result.push(c),
        }
    }
    result
}

/// Remove duplicates from an iterable while preserving order.
pub fn duplicates_removed<T: Eq + std::hash::Hash + Clone>(iter: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in iter {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

/// Extract the first paragraph/headline from a docstring.
pub fn docstring_headline(doc: &str) -> String {
    let trimmed = doc.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.len() == 1 {
        lines[0].trim().to_string()
    } else {
        // Collect lines until empty line or end
        let mut result = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            result.push(trimmed);
        }
        result.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bool_opt() {
        let mut opts = std::collections::HashMap::new();
        assert_eq!(get_bool_opt(&opts, "x", true).unwrap(), true);
        opts.insert("x".to_string(), "true".to_string());
        assert_eq!(get_bool_opt(&opts, "x", false).unwrap(), true);
        opts.insert("x".to_string(), "0".to_string());
        assert_eq!(get_bool_opt(&opts, "x", true).unwrap(), false);
    }

    #[test]
    fn test_get_int_opt() {
        let mut opts = std::collections::HashMap::new();
        assert_eq!(get_int_opt(&opts, "x", 42).unwrap(), 42);
        opts.insert("x".to_string(), "100".to_string());
        assert_eq!(get_int_opt(&opts, "x", 42).unwrap(), 100);
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<div>&\"test\"", true), "&lt;div&gt;&amp;&quot;test&quot;");
        assert_eq!(html_escape("<div>", false), "&lt;div&gt;");
    }

    #[test]
    fn test_looks_like_xml() {
        assert!(looks_like_xml("<?xml version='1.0'?>"));
        assert!(looks_like_xml("<div>test</div>"));
        assert!(!looks_like_xml("just plain text"));
    }

    #[test]
    fn test_shebang_matches() {
        assert!(shebang_matches("#!/usr/bin/python3", "python"));
        assert!(shebang_matches("#!/usr/bin/env python", "python"));
        assert!(!shebang_matches("#!/usr/bin/python3", "ruby"));
    }
}
