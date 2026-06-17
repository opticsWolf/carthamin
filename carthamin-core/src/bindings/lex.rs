use pyo3::prelude::*;
use crate::token::Token;
use crate::formatter::{Formatter, html::HtmlFormatter, terminal::TerminalFormatter, terminal256::{Terminal256Formatter, TerminalTrueColorFormatter}};
use crate::lexer::{Lexer, python::PythonLexer};
use crate::bindings::classes::PyToken;
use crate::registry::LexerRegistry;

/// Lex the given code with the given lexer.
/// Returns a list of (token_type, text) tuples.
#[pyfunction]
#[pyo3(name = "lex", signature = (code, lexer, _errstream=None))]
pub fn py_lex<'py>(
    py: Python<'py>,
    code: &str,
    lexer: &Bound<'py, pyo3::types::PyAny>,
    _errstream: Option<&Bound<'py, pyo3::types::PyAny>>,
) -> PyResult<Vec<(Py<PyToken>, String)>> {
    let lexer_name = lexer.str()?.extract::<String>()?;

    let registry = LexerRegistry::new();
    let tokens: Vec<(Token, String)> = if let Some(entry) = registry.get_by_alias(&lexer_name) {
        let lexer = (entry.create)();
        lexer.get_tokens(code)
    } else {
        // Fallback: try direct name matching
        match lexer_name.as_str() {
            "python" | "python3" | "py" => {
                let py_lexer = PythonLexer::new();
                py_lexer.get_tokens(code)
            }
            _ => {
                vec![(Token::TEXT, code.to_string())]
            }
        }
    };

    Ok(tokens.into_iter()
        .map(|(t, text)| (Py::new(py, PyToken(t)).unwrap(), text))
        .collect())
}

/// Format a token stream.
/// Returns the formatted output as a string.
#[pyfunction]
#[pyo3(name = "format", signature = (tokens, formatter, _outfile=None, noclasses=false))]
pub fn py_format(
    py: Python<'_>,
    tokens: Vec<(Py<PyToken>, String)>,
    formatter: &Bound<'_, pyo3::types::PyAny>,
    _outfile: Option<&Bound<'_, pyo3::types::PyAny>>,
    noclasses: bool,
) -> PyResult<String> {
    let parsed_tokens: Vec<(Token, String)> = tokens.into_iter()
        .map(|(py_token, text)| {
            let token = py_token.bind(py).borrow().0;
            (token, text)
        })
        .collect();

    let formatter_name = formatter.str()?.extract::<String>()?;

    let mut output = Vec::new();
    match formatter_name.as_str() {
        "html" | "HTML" => {
            let mut opts = crate::formatter::html::HtmlFormatterOptions::default();
            opts.noclasses = noclasses;
            let html_formatter = HtmlFormatter::new(Some(opts));
            html_formatter.format(&parsed_tokens, &mut output)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        }
        "terminal" | "console" | "tty" => {
            let term_formatter = TerminalFormatter::new(None);
            term_formatter.format(&parsed_tokens, &mut output)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        }
        "terminal256" | "256" => {
            let term256_formatter = Terminal256Formatter::new(None);
            term256_formatter.format(&parsed_tokens, &mut output)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        }
        _ => {
            for (_, text) in &parsed_tokens {
                output.extend_from_slice(text.as_bytes());
            }
        }
    }

    String::from_utf8(output).map_err(|e| {
        pyo3::exceptions::PyUnicodeDecodeError::new_err(e.to_string())
    })
}

/// Highlight code: lex + format in one step.
/// Returns the formatted output as a string.
#[pyfunction]
#[pyo3(name = "highlight", signature = (code, lexer, formatter, _outfile=None))]
pub fn py_highlight(
    py: Python<'_>,
    code: &str,
    lexer: &Bound<'_, pyo3::types::PyAny>,
    formatter: &Bound<'_, pyo3::types::PyAny>,
    _outfile: Option<&Bound<'_, pyo3::types::PyAny>>,
) -> PyResult<String> {
    let tokens = py_lex(py, code, lexer, None)?;
    py_format(py, tokens, formatter, None, false)
}
