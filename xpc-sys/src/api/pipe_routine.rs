use crate::object::try_xpc_into_rust::TryXPCIntoRust;
use crate::object::xpc_error::XPCError;
use crate::object::xpc_error::XPCError::PipeRoutineError;
use crate::object::xpc_object::{XPCHashMap, XPCObject};
use crate::{
    _xpc_pipe_interface_routine, get_xpc_bootstrap_pipe, rs_xpc_strerror, xpc_object_t,
    xpc_pipe_routine, xpc_pipe_routine_with_flags, xpc_pipe_t,
};
use std::ptr::null_mut;

fn check_error(errno: i32) -> Result<(), XPCError> {
    if errno != 0 {
        Err(PipeRoutineError(rs_xpc_strerror(errno)))
    } else {
        Ok(())
    }
}

pub fn pipe_routine<S: Into<XPCObject>>(
    xpc_pipe: Option<xpc_pipe_t>,
    dict: S,
) -> Result<XPCObject, XPCError> {
    let mut reply: xpc_object_t = null_mut();
    let pipe = xpc_pipe.unwrap_or(unsafe { get_xpc_bootstrap_pipe() });

    let errno = unsafe { xpc_pipe_routine(pipe, dict.into().as_ptr(), &mut reply) };
    check_error(errno)?;

    if reply.is_null() {
        Err(PipeRoutineError("reply was null".to_string()))
    } else {
        Ok(unsafe { XPCObject::from_raw(reply) })
    }
}

pub fn pipe_routine_with_flags<S: Into<XPCObject>>(
    xpc_pipe: Option<xpc_pipe_t>,
    dict: S,
    flags: Option<u64>,
) -> Result<XPCObject, XPCError> {
    let mut reply: xpc_object_t = null_mut();
    let pipe = xpc_pipe.unwrap_or(unsafe { get_xpc_bootstrap_pipe() });

    let errno = unsafe {
        xpc_pipe_routine_with_flags(pipe, dict.into().as_ptr(), &mut reply, flags.unwrap_or(0))
    };
    check_error(errno)?;

    if reply.is_null() {
        Err(PipeRoutineError("reply was null".to_string()))
    } else {
        Ok(unsafe { XPCObject::from_raw(reply) })
    }
}

pub fn pipe_interface_routine<S: Into<XPCObject>>(
    xpc_pipe: Option<xpc_pipe_t>,
    routine: u64,
    dict: S,
    flags: Option<u64>,
) -> Result<XPCObject, XPCError> {
    let mut reply: xpc_object_t = null_mut();
    let pipe = xpc_pipe.unwrap_or(unsafe { get_xpc_bootstrap_pipe() });

    let errno = unsafe {
        _xpc_pipe_interface_routine(
            pipe,
            routine,
            dict.into().as_ptr(),
            &mut reply,
            flags.unwrap_or(0),
        )
    };

    check_error(errno)?;

    if reply.is_null() {
        Err(PipeRoutineError("reply was null".to_string()))
    } else {
        Ok(unsafe { XPCObject::from_raw(reply) })
    }
}

pub fn handle_reply_dict_errors(reply: XPCObject) -> Result<XPCObject, XPCError> {
    let dict: XPCHashMap = reply.clone().to_rust()?;

    log::debug!("XPC dictionary reply {:?}", dict);

    if dict.contains_key("error") {
        let errcode: i64 = dict.get("error").unwrap().to_rust()?;

        Err(PipeRoutineError(format!(
            "{}: {}",
            errcode,
            rs_xpc_strerror(errcode as i32)
        )))
    } else if dict.contains_key("errors") {
        let errors_dict: XPCHashMap = dict.get("errors").unwrap().to_rust()?;

        if errors_dict.is_empty() {
            return Ok(reply);
        }

        let errors: Vec<String> = errors_dict
            .iter()
            .flat_map(|(_, e)| {
                let e: Result<i64, XPCError> = e.to_rust();
                e.map(|e_i64| format!("{}: {}", e_i64, rs_xpc_strerror(e_i64 as i32)))
            })
            .collect();

        Err(PipeRoutineError(errors.join("\n")))
    } else {
        Ok(reply)
    }
}
