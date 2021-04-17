use crate::objects::xpc_error::XPCError::{DictionaryError, PipeError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XPCError {
    DictionaryError(String),
    PipeError(String),
    ValueError(String),
    StandardError,
    NotFound,
}

impl Display for XPCError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            DictionaryError(e) => e,
            PipeError(e) => e,
            _ => "",
        };

        write!(f, "{}", err)
    }
}

impl Error for XPCError {}
