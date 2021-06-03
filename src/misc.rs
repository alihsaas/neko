use ansi_term::Colour;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum NekoError {
    SyntaxError(String),
    ReferenceError(String),
    TypeError(String),
    UnknownError(String),
}

impl Display for NekoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NekoError::SyntaxError(err) => {
                f.write_str(&format!("[{}]: {}", Colour::Red.paint("Syntax Error"), err))
            }
            NekoError::ReferenceError(err) => f.write_str(&format!(
                "[{}]: {}",
                Colour::Red.paint("Reference Error"),
                err
            )),
            NekoError::TypeError(err) => {
                f.write_str(&format!("[{}]: {}", Colour::Red.paint("Type Error"), err))
            }
            NekoError::UnknownError(err) => f.write_str(&format!(
                "[{}]: {}",
                Colour::Red.paint("Unknown Error"),
                err
            )),
        }
    }
}

impl From<String> for NekoError {
    fn from(string: String) -> Self {
        NekoError::UnknownError(string)
    }
}
