use std::convert::TryFrom;
use std::fmt;
use std::sync::Arc;

use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_object::XPCObject;
use crate::objects::xpc_type;
use crate::objects::xpc_type::check_xpc_type;
use crate::traits::xpc_value::TryXPCValue;

/// LimitLoadToSessionType key in XPC response
/// https://developer.apple.com/library/archive/technotes/tn2083/_index.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SessionType {
    Aqua = 0,
    StandardIO,
    Background,
    LoginWindow,
    System,
    Unknown,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<u64> for SessionType {
    fn from(session_type: u64) -> Self {
        match session_type {
            0 => SessionType::Aqua,
            1 => SessionType::StandardIO,
            2 => SessionType::Background,
            3 => SessionType::LoginWindow,
            4 => SessionType::System,
            _ => SessionType::Unknown,
        }
    }
}

// TODO: This feels terrible
impl From<String> for SessionType {
    fn from(value: String) -> Self {
        let aqua: String = SessionType::Aqua.to_string();
        let standard_io: String = SessionType::StandardIO.to_string();
        let background: String = SessionType::Background.to_string();
        let login_window: String = SessionType::LoginWindow.to_string();
        let system: String = SessionType::System.to_string();

        match value {
            s if s == aqua => SessionType::Aqua,
            s if s == standard_io => SessionType::StandardIO,
            s if s == background => SessionType::Background,
            s if s == login_window => SessionType::LoginWindow,
            s if s == system => SessionType::System,
            _ => SessionType::Unknown,
        }
    }
}

impl TryFrom<Arc<XPCObject>> for SessionType {
    type Error = XPCError;

    fn try_from(value: Arc<XPCObject>) -> Result<Self, Self::Error> {
        check_xpc_type(&*value, &xpc_type::String)?;
        let string: String = value.xpc_value().unwrap();
        Ok(string.into())
    }
}

// Huge thanks to: https://saelo.github.io/presentations/bits_of_launchd.pdf
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DomainType {
    System = 1,
    User = 2,
    UserLogin = 3,
    Session = 4,
    PID = 5,
    RequestorUserDomain = 6,
    RequestorDomain = 7,
    Unknown,
}

impl From<u64> for DomainType {
    fn from(value: u64) -> Self {
        match value {
            1 => DomainType::System,
            2 => DomainType::User,
            3 => DomainType::UserLogin,
            4 => DomainType::Session,
            5 => DomainType::PID,
            6 => DomainType::RequestorUserDomain,
            7 => DomainType::RequestorDomain,
            _ => DomainType::Unknown,
        }
    }
}

impl fmt::Display for DomainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
