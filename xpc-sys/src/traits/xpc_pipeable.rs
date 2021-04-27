use crate::objects::xpc_dictionary::XPCDictionary;
use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::PipeError;
use crate::objects::xpc_object::XPCObject;
use crate::{get_xpc_bootstrap_pipe, xpc_object_t, xpc_pipe_routine, xpc_pipe_routine_with_flags, rs_strerror};

use std::ptr::null_mut;

pub type XPCPipeResult = Result<XPCObject, XPCError>;

pub trait XPCPipeable {
    fn pipe_routine(&self) -> XPCPipeResult;
    fn pipe_routine_with_flags(&self, flags: u64) -> XPCPipeResult;

    fn handle_pipe_routine(ptr: xpc_object_t, errno: i32) -> XPCPipeResult {
        if errno == 0 {
            Ok(ptr.into())
        } else {
            Err(PipeError(rs_strerror(errno)))
        }
    }
}

impl XPCPipeable for XPCObject {
    fn pipe_routine(&self) -> XPCPipeResult {
        let XPCObject(arc, _) = self;
        let mut reply: xpc_object_t = null_mut();

        let err = unsafe { xpc_pipe_routine(get_xpc_bootstrap_pipe(), **arc, &mut reply) };

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
