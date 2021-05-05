use std::fmt;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::objects::xpc_type;
use xpc_sys::objects::xpc_type::check_xpc_type;
use xpc_sys::traits::xpc_value::TryXPCValue;
use xpc_sys::objects::xpc_error::XPCError;
use std::convert::TryFrom;

/// LimitLoadToSessionType key in XPC response
/// https://developer.apple.com/library/archive/technotes/tn2083/_index.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SessionType {
    Aqua,
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

impl TryFrom<XPCObject> for SessionType {
    type Error = XPCError;

    fn try_from(value: XPCObject) -> Result<Self, Self::Error> {
        check_xpc_type(&value, &xpc_type::String)?;
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
