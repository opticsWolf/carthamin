use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use crate::style::{Style, StyleAttributes};
use crate::style::generated::{self, ALL_STYLE_NAMES};
use crate::bindings::classes::PyToken;

/// Python-compatible StyleAttributes with property access.
#[pyclass(module = "carthamin")]
#[derive(Clone)]
pub struct PyStyleAttributes(pub StyleAttributes);

#[pymethods]
impl PyStyleAttributes {
    #[getter]
    fn color(&self) -> Option<String> {
        self.0.color.clone()
    }

    #[getter]
    fn bg(&self) -> Option<String> {
        self.0.bg.clone()
    }

    #[getter]
    fn bold(&self) -> Option<bool> {
        self.0.bold
    }

    #[getter]
    fn italic(&self) -> Option<bool> {
        self.0.italic
    }

    #[getter]
    fn underline(&self) -> Option<bool> {
        self.0.underline
    }

    #[getter]
    fn blink(&self) -> Option<bool> {
        self.0.blink
    }

    #[getter]
    fn roman(&self) -> Option<bool> {
        self.0.roman
    }

    fn __repr__(&self) -> String {
        let parts: Vec<String> = [
            self.0.color.as_ref().map(|c| format!("color:{}", c)),
            self.0.bg.as_ref().map(|b| format!("bg:{}", b)),
            self.0.bold.map(|v| if v { "bold".to_string() } else { "nobold".to_string() }),
            self.0.italic.map(|v| if v { "italic".to_string() } else { "noitalic".to_string() }),
            self.0.underline.map(|v| if v { "underline".to_string() } else { "nounderline".to_string() }),
        ]
        .into_iter()
        .flatten()
        .collect();
        format!("StyleAttributes({})", parts.join(", "))
    }

    /// Format as a CSS-style string (for comparison with Pygments).
    #[pyo3(name = "to_css_string")]
    fn css_string(&self) -> String {
        self.0.to_css_string()
    }
}

/// Python-compatible Style class.
/// Wraps the Rust Style struct and exposes Pygments-compatible API.
#[pyclass(module = "carthamin")]
pub struct PyStyle {
    inner: Style,
}

#[pymethods]
impl PyStyle {
    #[new]
    #[pyo3(signature = (name = "unnamed"))]
    fn new(name: &str) -> Self {
        PyStyle {
            inner: Style::new(name),
        }
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn background_color(&self) -> Option<String> {
        self.inner.default_style.bg.clone()
    }

    #[getter]
    fn highlight_color(&self) -> Option<String> {
        // Default highlight color if not overridden
        Some("#ffffcc".to_string())
    }

    /// Get the effective style for a token, walking up the inheritance chain.
    /// Returns a dict matching Pygments' style_for_token output format.
    #[pyo3(name = "style_for_token")]
    fn style_for_token_dict<'py>(&self, py: Python<'py>, token: &Bound<'py, PyToken>) -> PyResult<Bound<'py, PyDict>> {
        let attrs = self.inner.style_for_token(token.borrow().0);
        let d = PyDict::new(py);
        let _ = d.set_item("color", attrs.color.clone());
        let _ = d.set_item("bold", attrs.bold.unwrap_or(false));
        let _ = d.set_item("italic", attrs.italic.unwrap_or(false));
        let _ = d.set_item("underline", attrs.underline.unwrap_or(false));
        let _ = d.set_item("bgcolor", attrs.bg.clone());
        let _ = d.set_item("border", None::<Option<String>>);
        let _ = d.set_item("roman", attrs.roman.unwrap_or(false));
        let _ = d.set_item("sans", None::<Option<bool>>);
        let _ = d.set_item("mono", None::<Option<bool>>);
        Ok(d)
    }

    /// Get style attributes for a token (returns PyStyleAttributes).
    #[pyo3(name = "get_style_attributes")]
    fn get_style_attributes(&self, token: &Bound<'_, PyToken>) -> PyStyleAttributes {
        let attrs = self.inner.style_for_token(token.borrow().0).clone();
        PyStyleAttributes(attrs)
    }

    /// Check if a token has an explicit style definition.
    #[pyo3(name = "has_token_style")]
    fn has_token_style(&self, token: &Bound<'_, PyToken>) -> bool {
        self.inner.styles.contains_key(&token.borrow().0)
    }

    /// Iterate over all (token, style_dict) pairs.
    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let list = PyList::empty(py);
        for (_token, attrs) in &self.inner.styles {
            let py_attrs = PyStyleAttributes(attrs.clone());
            let py_obj = Py::new(py, py_attrs)?;
            let item = PyTuple::new(py, &[py_obj])?;
            list.append(item)?;
        }
        Ok(list)
    }

    fn __len__(&self) -> usize {
        self.inner.styles.len()
    }

    fn __repr__(&self) -> String {
        format!("Style('{}', {} entries)", self.inner.name, self.inner.styles.len())
    }
}

/// Get a style by name.
///
/// Returns a Style object for the given name, or raises ValueError if not found.
///
/// Available style names can be retrieved with ``get_all_styles()``.
#[pyfunction]
#[pyo3(name = "get_style_by_name")]
pub fn py_get_style_by_name(name: &str) -> PyResult<PyStyle> {
    match generated::get_style(name) {
        Some(style) => Ok(PyStyle { inner: style }),
        None => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "No style named '{}' found. Available styles: {}",
            name,
            ALL_STYLE_NAMES.join(", ")
        ))),
    }
}

/// Get a list of all available style names.
///
/// Returns a list of strings, each being a valid style name for ``get_style_by_name()``.
#[pyfunction]
#[pyo3(name = "get_all_styles")]
pub fn py_get_all_styles() -> Vec<&'static str> {
    ALL_STYLE_NAMES.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_style_monokai() {
        let style = py_get_style_by_name("monokai").unwrap();
        assert_eq!(style.inner.name, "monokai");
        assert_eq!(style.inner.default_style.bg, Some("#272822".to_string()));
    }

    #[test]
    fn test_py_style_invalid() {
        let result = py_get_style_by_name("nonexistent_style_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_py_all_styles() {
        let styles = py_get_all_styles();
        assert!(!styles.is_empty());
        assert!(styles.contains(&"monokai"));
        assert!(styles.contains(&"default"));
        assert!(styles.contains(&"dracula"));
    }
}
