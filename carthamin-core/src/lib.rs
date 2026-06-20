pub mod token;
pub mod regexopt;
pub mod util;
pub mod style;
pub mod scanner;
pub mod unistring;
pub mod lexer;
pub mod formatter;
pub mod filter;
pub mod registry;
pub mod bindings;

use pyo3::prelude::*;
use pyo3::create_exception;
use crate::bindings::classes::PyToken;
use crate::bindings::style::{PyStyle, PyStyleAttributes};
use crate::token::Token;

create_exception!(carthamin, ClassNotFound, pyo3::exceptions::PyValueError);
create_exception!(carthamin, OptionError, pyo3::exceptions::PyValueError);

/// Carthamin — A Rust reimplementation of Pygments.
#[pymodule]
fn carthamin(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Version
    m.add("__version__", "0.1.0")?;

    // Token class
    m.add_class::<PyToken>()?;

    // Style classes
    m.add_class::<PyStyle>()?;
    m.add_class::<PyStyleAttributes>()?;

    // Style functions
    m.add_function(wrap_pyfunction!(bindings::style::py_get_style_by_name, m)?)?;
    m.add_function(wrap_pyfunction!(bindings::style::py_get_all_styles, m)?)?;

    // Add known token constants to the module
    macro_rules! add_token {
        ($name:ident, $variant:ident) => {
            m.add(stringify!($name), PyToken(Token::$variant))?;
        };
    }

    add_token!(Token, TOKEN);
    add_token!(Text, TEXT);
    add_token!(Whitespace, WHITESPACE);
    add_token!(Escape, ESCAPE);
    add_token!(Error, ERROR);
    add_token!(Other, OTHER);
    add_token!(Keyword, KEYWORD);
    add_token!(Name, NAME);
    add_token!(Literal, LITERAL);
    add_token!(String, STRING);
    add_token!(Number, NUMBER);
    add_token!(Punctuation, PUNCTUATION);
    add_token!(Operator, OPERATOR);
    add_token!(Comment, COMMENT);
    add_token!(Generic, GENERIC);

    // Top-level functions
    m.add_function(wrap_pyfunction!(bindings::lex::py_lex, m)?)?;
    m.add_function(wrap_pyfunction!(bindings::lex::py_format, m)?)?;
    m.add_function(wrap_pyfunction!(bindings::lex::py_highlight, m)?)?;

    // Exceptions
    m.add("ClassNotFound", m.py().get_type::<ClassNotFound>())?;
    m.add("OptionError", m.py().get_type::<OptionError>())?;

    Ok(())
}
