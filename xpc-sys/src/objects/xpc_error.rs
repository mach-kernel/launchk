use crate::objects::xpc_error::XPCError::{DictionaryError, PipeError, QueryError, ShmemError, ValueError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XPCError {
    DictionaryError(String),
    PipeError(String),
    ValueError(String),
    QueryError(String),
    ShmemError(String),
    StandardError,
    NotFound,
}

impl Display for XPCError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            DictionaryError(e) => e,
            PipeError(e) => e,
            QueryError(e) => e,
            ValueError(e) => e,
            ShmemError(e) => e,
            _ => "",
        };

        write!(f, "{}", err)
    }
}

impl Error for XPCError {}
