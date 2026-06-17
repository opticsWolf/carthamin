use pyo3::prelude::*;
use crate::token::{Token, STANDARD_TYPES};

/// Python-compatible Token class with attribute access.
#[pyclass(module = "carthamin")]
#[derive(Clone, Copy)]
pub struct PyToken(pub Token);

#[pymethods]
impl PyToken {
    /// Get a subtoken by attribute name: Token.Keyword.Declaration
    fn __getattr__(&self, name: &str) -> PyResult<PyToken> {
        // Look up in known tokens
        for (t, path) in crate::token::ALL_TOKENS {
            if path.last() == Some(&name) && self.0.path.iter().copied().collect::<Vec<_>>() == &path[..path.len().saturating_sub(1)] {
                return Ok(PyToken(*t));
            }
        }
        Err(pyo3::exceptions::PyAttributeError::new_err(format!(
            "Token.{}.{} is not a known token type", self.0.to_string_full(), name
        )))
    }

    fn __repr__(&self) -> String {
        self.0.to_string_full()
    }

    fn __str__(&self) -> String {
        self.0.to_string_full()
    }

    fn __eq__(&self, other: &PyToken) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> u64 {
        let mut hash: u64 = 0;
        for part in self.0.path {
            hash = hash.wrapping_mul(31).wrapping_add(part.as_bytes().iter().map(|b| *b as u64).sum::<u64>());
        }
        hash
    }

    fn __contains__(&self, other: &PyToken) -> bool {
        other.0.is_subtype_of(&self.0) || other.0 == self.0
    }

    /// Check if this token is a subtype of another token.
    #[pyo3(name = "is_subtype_of")]
    fn is_subtype(&self, other: &PyToken) -> bool {
        self.0.is_subtype_of(&other.0)
    }

    /// Get the CSS class name for this token.
    #[pyo3(name = "string")]
    fn css_class(&self) -> &'static str {
        STANDARD_TYPES.get(&self.0).copied().unwrap_or("")
    }

    /// Split into path components.
    pub fn split(&self) -> Vec<String> {
        let mut parts = vec!["Token".to_string()];
        for part in self.0.path {
            parts.push(part.to_string());
        }
        parts
    }
}
