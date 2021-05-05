use crate::objects::xpc_dictionary::XPCDictionary;
use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::PipeError;
use crate::objects::xpc_object::XPCObject;
use crate::{
    get_xpc_bootstrap_pipe, rs_strerror, rs_xpc_strerror, xpc_object_t, xpc_pipe_routine,
    xpc_pipe_routine_with_flags,
};

use crate::traits::xpc_value::TryXPCValue;
use std::convert::TryInto;
use std::ptr::null_mut;

pub type XPCPipeResult = Result<XPCObject, XPCError>;

pub trait XPCPipeable {
    fn pipe_routine(&self) -> XPCPipeResult;
    fn pipe_routine_with_flags(&self, flags: u64) -> XPCPipeResult;

    /// Pipe routine expecting XPC dictionary reply, with checking of "error" and "errors" keys
    fn pipe_routine_with_error_handling(&self) -> Result<XPCDictionary, XPCError> {
        let response = self.pipe_routine()?.try_into()?;
        let XPCDictionary(hm) = &response;

        if hm.contains_key("error") {
            let errcode: i64 = response.get(&["error"])?.xpc_value()?;
            Err(XPCError::QueryError(format!(
                "{}: {}",
                errcode,
                rs_xpc_strerror(errcode as i32)
            )))
        } else if hm.contains_key("errors") {
            let XPCDictionary(errors_hm) = response.get_as_dictionary(&["errors"])?;
            if errors_hm.is_empty() {
                return Ok(response);
            }

            let errors: Vec<String> = errors_hm
                .iter()
                .flat_map(|(_, e)| {
                    let e: Result<i64, XPCError> = e.xpc_value();
                    e.map(|e_i64| format!("{}: {}", e_i64, rs_xpc_strerror(e_i64 as i32)))
                })
                .collect();

            Err(XPCError::QueryError(errors.join("\n")))
        } else {
            Ok(response)
        }
    }

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
