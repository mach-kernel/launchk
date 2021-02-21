use actix::prelude::*;
use actix::{Actor, Addr, Context, System, Handler};
use xpc_sys::{xpc_object_t, get_xpc_bootstrap_pipe, xpc_pipe_routine, xpc_pipe_routine_with_flags, xpc_pipe_t, XPCObject};
use std::ptr::null_mut;
use std::os::raw::c_int;

use crate::actor::xpc::XPCRequestError::{PipeNotInitialized, XPCPipeRoutineError};
use crate::actor::xpc::XPCRequest::{PipeRoutine, PipeRoutineWithFlags};

#[derive(Default)]
pub struct XPCActor {
    pipe: Option<xpc_pipe_t>
}

#[derive(Message)]
#[rtype(result = "Result<XPCObject, XPCRequestError>")]
pub enum XPCRequest {
    PipeRoutine(xpc_object_t),
    PipeRoutineWithFlags(xpc_object_t, u64)
}

pub enum XPCRequestError {
    PipeNotInitialized,
    XPCPipeRoutineError(c_int)
}

impl Actor for XPCActor {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        self.pipe = Some(get_xpc_bootstrap_pipe())
    }
}

impl Handler<XPCRequest> for XPCActor {
    type Result = Result<XPCObject, XPCRequestError>;

    fn handle(&mut self, msg: XPCRequest, ctx: &mut Context<Self>) -> Self::Result {
        if self.pipe.is_none() {
            return Err(PipeNotInitialized)
        }

        let mut reply: xpc_object_t = null_mut();

        let err = unsafe {
            match msg {
                PipeRoutine(payload) => xpc_pipe_routine(self.pipe.unwrap(), payload, &mut reply),
                PipeRoutineWithFlags(payload, flags) => xpc_pipe_routine_with_flags(self.pipe.unwrap(), payload, &mut reply, flags)
            }
        };

        if err == 0 {
            Ok(XPCObject { data: reply })
        } else {
            Err(XPCPipeRoutineError(err))
        }
    }
}
