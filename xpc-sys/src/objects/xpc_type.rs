use std::collections::HashMap;

use crate::{mach_port_t, xpc_get_type, xpc_object_t, xpc_type_t};

use crate::objects::xpc_object::XPCObject;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XPCType(pub xpc_type_t);

unsafe impl Send for XPCType {}
unsafe impl Sync for XPCType {}

impl From<xpc_object_t> for XPCType {
    fn from(value: xpc_object_t) -> XPCType {
        XPCType(unsafe { xpc_get_type(value) })
    }
}

impl From<XPCObject> for XPCType {
    fn from(value: XPCObject) -> XPCType {
        let XPCObject(_, xpc_type) = value;
        xpc_type
    }
}

// TODO: Why is this broken? Even with lazy_static!
// impl From<_xpc_type_s> for XPCType {
//     /// Mostly for use with externs like _xpc_type_dictionary
//     /// from bindgen
//     fn from(value: _xpc_type_s) -> XPCType {
//         let xpc_type: xpc_type_t = &value;
//         XPCType(xpc_type)
//     }
// }
// pub static ref Dictionary: XPCType = unsafe { _xpc_type_dictionary.into() };

/*
   All _xpc_type_* from bindings.rs

   pub static _xpc_bool_true: _xpc_bool_s;
   pub static _xpc_bool_false: _xpc_bool_s;
   pub static _xpc_type_int64: _xpc_type_s;
   pub static _xpc_type_uint64: _xpc_type_s;
   pub static _xpc_type_double: _xpc_type_s;
   pub static _xpc_type_date: _xpc_type_s;
   pub static _xpc_type_data: _xpc_type_s;
   pub static _xpc_type_string: _xpc_type_s;
   pub static _xpc_type_uuid: _xpc_type_s;
   pub static _xpc_type_fd: _xpc_type_s;
   pub static _xpc_type_shmem: _xpc_type_s;
   pub static _xpc_type_array: _xpc_type_s;
   pub static _xpc_type_dictionary: _xpc_type_s;
   pub static _xpc_type_error: _xpc_type_s;
*/

lazy_static! {
    pub static ref Dictionary: XPCType = {
        let empty: HashMap<&str, XPCObject> = HashMap::new();
        XPCObject::from(empty).xpc_type()
    };
    pub static ref Int64: XPCType = XPCObject::from(0 as i64).into();
    pub static ref UInt64: XPCType = XPCObject::from(0 as u64).into();
    pub static ref Double: XPCType = XPCObject::from(2.0 as f64).into();
    pub static ref String: XPCType = XPCObject::from("").into();
    pub static ref Bool: XPCType = XPCObject::from(true).into();
    pub static ref MachPort: XPCType = XPCObject::from(0 as mach_port_t).into();
}
