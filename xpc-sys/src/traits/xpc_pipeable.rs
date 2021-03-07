use std::fmt::{Display, Formatter};
use crate::{xpc_object_t, get_bootstrap_port, xpc_pipe_routine, get_xpc_bootstrap_pipe, str_errno, xpc_pipe_routine_with_flags};
use crate::objects::xpc_object::XPCObject;
use std::ptr::null_mut;
use std::error::Error;
use crate::objects::xpc_dictionary::XPCDictionary;

#[derive(Debug)]
pub struct XPCPipeError(pub String);

impl Display for XPCPipeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for XPCPipeError {}

pub type XPCPipeResult = Result<XPCObject, XPCPipeError>;

pub trait XPCPipeable {
    fn pipe_routine(&self) -> XPCPipeResult;
    fn pipe_routine_with_flags(&self, flags: u64) -> XPCPipeResult;

    fn handle_pipe_routine(ptr: xpc_object_t, errno: i32) -> XPCPipeResult {
        if errno == 0 {
            Ok(ptr.into())
        } else {
            Err(XPCPipeError(str_errno(Some(errno))))
        }
    }
}

impl XPCPipeable for XPCObject {
    fn pipe_routine(&self) -> XPCPipeResult {
        let XPCObject(arc, _) = self;
        let mut reply: xpc_object_t = null_mut();

        let err = unsafe {
            xpc_pipe_routine(get_xpc_bootstrap_pipe(), **arc, &mut reply)
        };

        Self::handle_pipe_routine(reply, err)
    }

    fn pipe_routine_with_flags(&self, flags: u64) -> XPCPipeResult {
        let XPCObject(arc, _) = self;
        let mut reply: xpc_object_t = null_mut();

        let err = unsafe {
            xpc_pipe_routine_with_flags(get_xpc_bootstrap_pipe(), **arc, &mut reply, flags)
        };

        Self::handle_pipe_routine(reply, err)
    }
}

impl XPCPipeable for XPCDictionary {
    fn pipe_routine(&self) -> XPCPipeResult {
        let xpc_object: XPCObject = self.into();
        xpc_object.pipe_routine()
    }

    fn pipe_routine_with_flags(&self, flags: u64) -> XPCPipeResult {
        let xpc_object: XPCObject = self.into();
        xpc_object.pipe_routine_with_flags(flags)
    }
}