use crate::xpc_pipe_t;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
pub struct XPCPipe(pub xpc_pipe_t);

unsafe impl Send for XPCPipe {}
unsafe impl Sync for XPCPipe {}
