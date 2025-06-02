use crate::object::xpc_error::XPCError::{
    DictionaryError, IOError, PipeRoutineError, ValueError,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XPCError {
    DictionaryError(String),
    ValueError(String),
    PipeRoutineError(String),
    IOError(String),
    NotFound,
}

impl Display for XPCError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            DictionaryError(e) => e,
            PipeRoutineError(e) => e,
            ValueError(e) => e,
            IOError(e) => e,
            _ => "",
        };

        write!(f, "{}", err)
    }
}

impl Error for XPCError {}
