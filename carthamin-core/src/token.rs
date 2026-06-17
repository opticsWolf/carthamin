use std::fmt;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// A token type in the Carthamin token tree.
/// Mirrors Pygments' _TokenType hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Token {
    /// Path components from root (Token) to this node.
    /// e.g., Token.Keyword.Declaration = ["Keyword", "Declaration"]
    pub path: &'static [&'static str],
}

// We use a flat representation for simplicity.
// Each variant is a &'static [ &'static str ] path.

impl Token {
    /// Root token (Token itself)
    pub const TOKEN: Token = Token { path: &[] };

    // Special
    pub const TEXT: Token = Token { path: &["Text"] };
    pub const WHITESPACE: Token = Token { path: &["Text", "Whitespace"] };
    pub const ESCAPE: Token = Token { path: &["Escape"] };
    pub const ERROR: Token = Token { path: &["Error"] };
    pub const OTHER: Token = Token { path: &["Other"] };

    // Common
    pub const KEYWORD: Token = Token { path: &["Keyword"] };
    pub const KEYWORD_CONSTANT: Token = Token { path: &["Keyword", "Constant"] };
    pub const KEYWORD_DECLARATION: Token = Token { path: &["Keyword", "Declaration"] };
    pub const KEYWORD_NAMESPACE: Token = Token { path: &["Keyword", "Namespace"] };
    pub const KEYWORD_PSEUDO: Token = Token { path: &["Keyword", "Pseudo"] };
    pub const KEYWORD_RESERVED: Token = Token { path: &["Keyword", "Reserved"] };
    pub const KEYWORD_TYPE: Token = Token { path: &["Keyword", "Type"] };

    pub const NAME: Token = Token { path: &["Name"] };
    pub const NAME_ATTRIBUTE: Token = Token { path: &["Name", "Attribute"] };
    pub const NAME_BUILTIN: Token = Token { path: &["Name", "Builtin"] };
    pub const NAME_BUILTIN_PSEUDO: Token = Token { path: &["Name", "Builtin", "Pseudo"] };
    pub const NAME_CLASS: Token = Token { path: &["Name", "Class"] };
    pub const NAME_CONSTANT: Token = Token { path: &["Name", "Constant"] };
    pub const NAME_DECORATOR: Token = Token { path: &["Name", "Decorator"] };
    pub const NAME_ENTITY: Token = Token { path: &["Name", "Entity"] };
    pub const NAME_EXCEPTION: Token = Token { path: &["Name", "Exception"] };
    pub const NAME_FUNCTION: Token = Token { path: &["Name", "Function"] };
    pub const NAME_FUNCTION_MAGIC: Token = Token { path: &["Name", "Function", "Magic"] };
    pub const NAME_PROPERTY: Token = Token { path: &["Name", "Property"] };
    pub const NAME_LABEL: Token = Token { path: &["Name", "Label"] };
    pub const NAME_NAMESPACE: Token = Token { path: &["Name", "Namespace"] };
    pub const NAME_OTHER: Token = Token { path: &["Name", "Other"] };
    pub const NAME_TAG: Token = Token { path: &["Name", "Tag"] };
    pub const NAME_VARIABLE: Token = Token { path: &["Name", "Variable"] };
    pub const NAME_VARIABLE_CLASS: Token = Token { path: &["Name", "Variable", "Class"] };
    pub const NAME_VARIABLE_GLOBAL: Token = Token { path: &["Name", "Variable", "Global"] };
    pub const NAME_VARIABLE_INSTANCE: Token = Token { path: &["Name", "Variable", "Instance"] };
    pub const NAME_VARIABLE_MAGIC: Token = Token { path: &["Name", "Variable", "Magic"] };

    pub const LITERAL: Token = Token { path: &["Literal"] };
    pub const LITERAL_DATE: Token = Token { path: &["Literal", "Date"] };

    pub const STRING: Token = Token { path: &["Literal", "String"] };
    pub const STRING_AFFIX: Token = Token { path: &["Literal", "String", "Affix"] };
    pub const STRING_BACKTICK: Token = Token { path: &["Literal", "String", "Backtick"] };
    pub const STRING_CHAR: Token = Token { path: &["Literal", "String", "Char"] };
    pub const STRING_DELIMITER: Token = Token { path: &["Literal", "String", "Delimiter"] };
    pub const STRING_DOC: Token = Token { path: &["Literal", "String", "Doc"] };
    pub const STRING_DOUBLE: Token = Token { path: &["Literal", "String", "Double"] };
    pub const STRING_ESCAPE: Token = Token { path: &["Literal", "String", "Escape"] };
    pub const STRING_HEREDOC: Token = Token { path: &["Literal", "String", "Heredoc"] };
    pub const STRING_INTERPOL: Token = Token { path: &["Literal", "String", "Interpol"] };
    pub const STRING_OTHER: Token = Token { path: &["Literal", "String", "Other"] };
    pub const STRING_REGEX: Token = Token { path: &["Literal", "String", "Regex"] };
    pub const STRING_SINGLE: Token = Token { path: &["Literal", "String", "Single"] };
    pub const STRING_SYMBOL: Token = Token { path: &["Literal", "String", "Symbol"] };

    pub const NUMBER: Token = Token { path: &["Literal", "Number"] };
    pub const NUMBER_BIN: Token = Token { path: &["Literal", "Number", "Bin"] };
    pub const NUMBER_FLOAT: Token = Token { path: &["Literal", "Number", "Float"] };
    pub const NUMBER_HEX: Token = Token { path: &["Literal", "Number", "Hex"] };
    pub const NUMBER_INTEGER: Token = Token { path: &["Literal", "Number", "Integer"] };
    pub const NUMBER_INTEGER_LONG: Token = Token { path: &["Literal", "Number", "Integer", "Long"] };
    pub const NUMBER_OCT: Token = Token { path: &["Literal", "Number", "Oct"] };

    pub const OPERATOR: Token = Token { path: &["Operator"] };
    pub const OPERATOR_WORD: Token = Token { path: &["Operator", "Word"] };

    pub const PUNCTUATION: Token = Token { path: &["Punctuation"] };
    pub const PUNCTUATION_MARKER: Token = Token { path: &["Punctuation", "Marker"] };

    pub const COMMENT: Token = Token { path: &["Comment"] };
    pub const COMMENT_HASHBANG: Token = Token { path: &["Comment", "Hashbang"] };
    pub const COMMENT_MULTILINE: Token = Token { path: &["Comment", "Multiline"] };
    pub const COMMENT_PREPROC: Token = Token { path: &["Comment", "Preproc"] };
    pub const COMMENT_PREPROCFILE: Token = Token { path: &["Comment", "PreprocFile"] };
    pub const COMMENT_SINGLE: Token = Token { path: &["Comment", "Single"] };
    pub const COMMENT_SPECIAL: Token = Token { path: &["Comment", "Special"] };

    pub const GENERIC: Token = Token { path: &["Generic"] };
    pub const GENERIC_DELETED: Token = Token { path: &["Generic", "Deleted"] };
    pub const GENERIC_EMPH: Token = Token { path: &["Generic", "Emph"] };
    pub const GENERIC_ERROR: Token = Token { path: &["Generic", "Error"] };
    pub const GENERIC_HEADING: Token = Token { path: &["Generic", "Heading"] };
    pub const GENERIC_INSERTED: Token = Token { path: &["Generic", "Inserted"] };
    pub const GENERIC_OUTPUT: Token = Token { path: &["Generic", "Output"] };
    pub const GENERIC_PROMPT: Token = Token { path: &["Generic", "Prompt"] };
    pub const GENERIC_STRONG: Token = Token { path: &["Generic", "Strong"] };
    pub const GENERIC_SUBHEADING: Token = Token { path: &["Generic", "Subheading"] };
    pub const GENERIC_EMPH_STRONG: Token = Token { path: &["Generic", "EmphStrong"] };
    pub const GENERIC_TRACEBACK: Token = Token { path: &["Generic", "Traceback"] };

    /// Check if this token is a subtype of `other`.
    /// Token.Keyword.Declaration in Token.Keyword → true
    pub fn is_subtype_of(&self, other: &Token) -> bool {
        self == other || self.path.starts_with(other.path)
    }

    /// Get the parent token, or TOKEN if at root.
    pub fn parent(&self) -> Option<Token> {
        if self.path.is_empty() {
            None
        } else {
            let _parent_path: Vec<&str> = self.path[..self.path.len() - 1].to_vec();
            // For now, just return TOKEN as fallback for simplicity
            Some(Token::TOKEN)
        }
    }

    /// Convert to dot-notation string: "Token.Keyword.Declaration"
    pub fn to_string_full(&self) -> String {
        if self.path.is_empty() {
            "Token".to_string()
        } else {
            format!("Token.{}", self.path.join("."))
        }
    }

    /// Parse from dot-notation string: "Token.Keyword.Declaration" or "Keyword.Declaration"
    pub fn from_string(s: &str) -> Option<Token> {
        let s = if s.starts_with("Token.") {
            &s[6..]
        } else {
            s
        };
        if s.is_empty() {
            return Some(Token::TOKEN);
        }
        // Look up in known tokens
        let parts: Vec<&str> = s.split('.').collect();
        ALL_TOKENS.iter().find(|(_, path)| {
            *path == parts.as_slice()
        }).map(|(t, _)| *t)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_full())
    }
}

/// CSS class name mapping for standard tokens.
pub static STANDARD_TYPES: Lazy<HashMap<Token, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(Token::TOKEN, "");
    m.insert(Token::TEXT, "");
    m.insert(Token::WHITESPACE, "w");
    m.insert(Token::ESCAPE, "esc");
    m.insert(Token::ERROR, "err");
    m.insert(Token::OTHER, "x");
    m.insert(Token::KEYWORD, "k");
    m.insert(Token::KEYWORD_CONSTANT, "kc");
    m.insert(Token::KEYWORD_DECLARATION, "kd");
    m.insert(Token::KEYWORD_NAMESPACE, "kn");
    m.insert(Token::KEYWORD_PSEUDO, "kp");
    m.insert(Token::KEYWORD_RESERVED, "kr");
    m.insert(Token::KEYWORD_TYPE, "kt");
    m.insert(Token::NAME, "n");
    m.insert(Token::NAME_ATTRIBUTE, "na");
    m.insert(Token::NAME_BUILTIN, "nb");
    m.insert(Token::NAME_BUILTIN_PSEUDO, "bp");
    m.insert(Token::NAME_CLASS, "nc");
    m.insert(Token::NAME_CONSTANT, "no");
    m.insert(Token::NAME_DECORATOR, "nd");
    m.insert(Token::NAME_ENTITY, "ni");
    m.insert(Token::NAME_EXCEPTION, "ne");
    m.insert(Token::NAME_FUNCTION, "nf");
    m.insert(Token::NAME_FUNCTION_MAGIC, "fm");
    m.insert(Token::NAME_PROPERTY, "py");
    m.insert(Token::NAME_LABEL, "nl");
    m.insert(Token::NAME_NAMESPACE, "nn");
    m.insert(Token::NAME_OTHER, "nx");
    m.insert(Token::NAME_TAG, "nt");
    m.insert(Token::NAME_VARIABLE, "nv");
    m.insert(Token::NAME_VARIABLE_CLASS, "vc");
    m.insert(Token::NAME_VARIABLE_GLOBAL, "vg");
    m.insert(Token::NAME_VARIABLE_INSTANCE, "vi");
    m.insert(Token::NAME_VARIABLE_MAGIC, "vm");
    m.insert(Token::LITERAL, "l");
    m.insert(Token::LITERAL_DATE, "ld");
    m.insert(Token::STRING, "s");
    m.insert(Token::STRING_AFFIX, "sa");
    m.insert(Token::STRING_BACKTICK, "sb");
    m.insert(Token::STRING_CHAR, "sc");
    m.insert(Token::STRING_DELIMITER, "dl");
    m.insert(Token::STRING_DOC, "sd");
    m.insert(Token::STRING_DOUBLE, "s2");
    m.insert(Token::STRING_ESCAPE, "se");
    m.insert(Token::STRING_HEREDOC, "sh");
    m.insert(Token::STRING_INTERPOL, "si");
    m.insert(Token::STRING_OTHER, "sx");
    m.insert(Token::STRING_REGEX, "sr");
    m.insert(Token::STRING_SINGLE, "s1");
    m.insert(Token::STRING_SYMBOL, "ss");
    m.insert(Token::NUMBER, "m");
    m.insert(Token::NUMBER_BIN, "mb");
    m.insert(Token::NUMBER_FLOAT, "mf");
    m.insert(Token::NUMBER_HEX, "mh");
    m.insert(Token::NUMBER_INTEGER, "mi");
    m.insert(Token::NUMBER_INTEGER_LONG, "il");
    m.insert(Token::NUMBER_OCT, "mo");
    m.insert(Token::OPERATOR, "o");
    m.insert(Token::OPERATOR_WORD, "ow");
    m.insert(Token::PUNCTUATION, "p");
    m.insert(Token::PUNCTUATION_MARKER, "pm");
    m.insert(Token::COMMENT, "c");
    m.insert(Token::COMMENT_HASHBANG, "ch");
    m.insert(Token::COMMENT_MULTILINE, "cm");
    m.insert(Token::COMMENT_PREPROC, "cp");
    m.insert(Token::COMMENT_PREPROCFILE, "cpf");
    m.insert(Token::COMMENT_SINGLE, "c1");
    m.insert(Token::COMMENT_SPECIAL, "cs");
    m.insert(Token::GENERIC, "g");
    m.insert(Token::GENERIC_DELETED, "gd");
    m.insert(Token::GENERIC_EMPH, "ge");
    m.insert(Token::GENERIC_ERROR, "gr");
    m.insert(Token::GENERIC_HEADING, "gh");
    m.insert(Token::GENERIC_INSERTED, "gi");
    m.insert(Token::GENERIC_OUTPUT, "go");
    m.insert(Token::GENERIC_PROMPT, "gp");
    m.insert(Token::GENERIC_STRONG, "gs");
    m.insert(Token::GENERIC_SUBHEADING, "gu");
    m.insert(Token::GENERIC_EMPH_STRONG, "ges");
    m.insert(Token::GENERIC_TRACEBACK, "gt");
    m
});

/// All known tokens for lookup.
pub static ALL_TOKENS: &[(Token, &[&str])] = &[
    (Token::TOKEN, &[]),
    (Token::TEXT, &["Text"]),
    (Token::WHITESPACE, &["Text", "Whitespace"]),
    (Token::ESCAPE, &["Escape"]),
    (Token::ERROR, &["Error"]),
    (Token::OTHER, &["Other"]),
    (Token::KEYWORD, &["Keyword"]),
    (Token::KEYWORD_CONSTANT, &["Keyword", "Constant"]),
    (Token::KEYWORD_DECLARATION, &["Keyword", "Declaration"]),
    (Token::KEYWORD_NAMESPACE, &["Keyword", "Namespace"]),
    (Token::KEYWORD_PSEUDO, &["Keyword", "Pseudo"]),
    (Token::KEYWORD_RESERVED, &["Keyword", "Reserved"]),
    (Token::KEYWORD_TYPE, &["Keyword", "Type"]),
    (Token::NAME, &["Name"]),
    (Token::NAME_ATTRIBUTE, &["Name", "Attribute"]),
    (Token::NAME_BUILTIN, &["Name", "Builtin"]),
    (Token::NAME_BUILTIN_PSEUDO, &["Name", "Builtin", "Pseudo"]),
    (Token::NAME_CLASS, &["Name", "Class"]),
    (Token::NAME_CONSTANT, &["Name", "Constant"]),
    (Token::NAME_DECORATOR, &["Name", "Decorator"]),
    (Token::NAME_ENTITY, &["Name", "Entity"]),
    (Token::NAME_EXCEPTION, &["Name", "Exception"]),
    (Token::NAME_FUNCTION, &["Name", "Function"]),
    (Token::NAME_FUNCTION_MAGIC, &["Name", "Function", "Magic"]),
    (Token::NAME_PROPERTY, &["Name", "Property"]),
    (Token::NAME_LABEL, &["Name", "Label"]),
    (Token::NAME_NAMESPACE, &["Name", "Namespace"]),
    (Token::NAME_OTHER, &["Name", "Other"]),
    (Token::NAME_TAG, &["Name", "Tag"]),
    (Token::NAME_VARIABLE, &["Name", "Variable"]),
    (Token::NAME_VARIABLE_CLASS, &["Name", "Variable", "Class"]),
    (Token::NAME_VARIABLE_GLOBAL, &["Name", "Variable", "Global"]),
    (Token::NAME_VARIABLE_INSTANCE, &["Name", "Variable", "Instance"]),
    (Token::NAME_VARIABLE_MAGIC, &["Name", "Variable", "Magic"]),
    (Token::LITERAL, &["Literal"]),
    (Token::LITERAL_DATE, &["Literal", "Date"]),
    (Token::STRING, &["Literal", "String"]),
    (Token::STRING_AFFIX, &["Literal", "String", "Affix"]),
    (Token::STRING_BACKTICK, &["Literal", "String", "Backtick"]),
    (Token::STRING_CHAR, &["Literal", "String", "Char"]),
    (Token::STRING_DELIMITER, &["Literal", "String", "Delimiter"]),
    (Token::STRING_DOC, &["Literal", "String", "Doc"]),
    (Token::STRING_DOUBLE, &["Literal", "String", "Double"]),
    (Token::STRING_ESCAPE, &["Literal", "String", "Escape"]),
    (Token::STRING_HEREDOC, &["Literal", "String", "Heredoc"]),
    (Token::STRING_INTERPOL, &["Literal", "String", "Interpol"]),
    (Token::STRING_OTHER, &["Literal", "String", "Other"]),
    (Token::STRING_REGEX, &["Literal", "String", "Regex"]),
    (Token::STRING_SINGLE, &["Literal", "String", "Single"]),
    (Token::STRING_SYMBOL, &["Literal", "String", "Symbol"]),
    (Token::NUMBER, &["Literal", "Number"]),
    (Token::NUMBER_BIN, &["Literal", "Number", "Bin"]),
    (Token::NUMBER_FLOAT, &["Literal", "Number", "Float"]),
    (Token::NUMBER_HEX, &["Literal", "Number", "Hex"]),
    (Token::NUMBER_INTEGER, &["Literal", "Number", "Integer"]),
    (Token::NUMBER_INTEGER_LONG, &["Literal", "Number", "Integer", "Long"]),
    (Token::NUMBER_OCT, &["Literal", "Number", "Oct"]),
    (Token::OPERATOR, &["Operator"]),
    (Token::OPERATOR_WORD, &["Operator", "Word"]),
    (Token::PUNCTUATION, &["Punctuation"]),
    (Token::PUNCTUATION_MARKER, &["Punctuation", "Marker"]),
    (Token::COMMENT, &["Comment"]),
    (Token::COMMENT_HASHBANG, &["Comment", "Hashbang"]),
    (Token::COMMENT_MULTILINE, &["Comment", "Multiline"]),
    (Token::COMMENT_PREPROC, &["Comment", "Preproc"]),
    (Token::COMMENT_PREPROCFILE, &["Comment", "PreprocFile"]),
    (Token::COMMENT_SINGLE, &["Comment", "Single"]),
    (Token::COMMENT_SPECIAL, &["Comment", "Special"]),
    (Token::GENERIC, &["Generic"]),
    (Token::GENERIC_DELETED, &["Generic", "Deleted"]),
    (Token::GENERIC_EMPH, &["Generic", "Emph"]),
    (Token::GENERIC_ERROR, &["Generic", "Error"]),
    (Token::GENERIC_HEADING, &["Generic", "Heading"]),
    (Token::GENERIC_INSERTED, &["Generic", "Inserted"]),
    (Token::GENERIC_OUTPUT, &["Generic", "Output"]),
    (Token::GENERIC_PROMPT, &["Generic", "Prompt"]),
    (Token::GENERIC_STRONG, &["Generic", "Strong"]),
    (Token::GENERIC_SUBHEADING, &["Generic", "Subheading"]),
    (Token::GENERIC_EMPH_STRONG, &["Generic", "EmphStrong"]),
    (Token::GENERIC_TRACEBACK, &["Generic", "Traceback"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_subtype() {
        assert!(Token::KEYWORD_DECLARATION.is_subtype_of(&Token::KEYWORD));
        assert!(Token::KEYWORD_DECLARATION.is_subtype_of(&Token::KEYWORD_DECLARATION));
        assert!(!Token::KEYWORD_DECLARATION.is_subtype_of(&Token::NAME));
        assert!(Token::STRING_DOUBLE.is_subtype_of(&Token::STRING));
        assert!(Token::STRING_DOUBLE.is_subtype_of(&Token::LITERAL));
    }

    #[test]
    fn test_token_display() {
        assert_eq!(Token::TOKEN.to_string(), "Token");
        assert_eq!(Token::KEYWORD_DECLARATION.to_string(), "Token.Keyword.Declaration");
        assert_eq!(Token::STRING_DOUBLE.to_string(), "Token.Literal.String.Double");
    }

    #[test]
    fn test_standard_types() {
        assert_eq!(STANDARD_TYPES.get(&Token::WHITESPACE), Some(&"w"));
        assert_eq!(STANDARD_TYPES.get(&Token::KEYWORD_DECLARATION), Some(&"kd"));
        assert_eq!(STANDARD_TYPES.get(&Token::STRING_DOUBLE), Some(&"s2"));
    }

    #[test]
    fn test_from_string() {
        assert_eq!(Token::from_string("Keyword.Declaration"), Some(Token::KEYWORD_DECLARATION));
        assert_eq!(Token::from_string("Token.Keyword.Declaration"), Some(Token::KEYWORD_DECLARATION));
        assert_eq!(Token::from_string(""), Some(Token::TOKEN));
    }
}
