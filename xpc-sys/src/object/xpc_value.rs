use crate::object::xpc_object::XPCObject;
use crate::{
    mach_port_t, xpc_bool_get_value, xpc_int64_get_value, xpc_object_t, xpc_string_get_string_ptr,
    xpc_uint64_get_value,
};
use std::ffi::CStr;

pub trait XPCValue<Out> {
    fn xpc_value(&self) -> Out;
}

impl XPCValue<i64> for XPCObject {
    fn xpc_value(&self) -> i64 {
        let XPCObject(obj_pointer, _) = self;
        unsafe { xpc_int64_get_value(**obj_pointer) }
    }
}

impl XPCValue<u64> for XPCObject {
    fn xpc_value(&self) -> u64 {
        let XPCObject(obj_pointer, _) = self;
        unsafe { xpc_uint64_get_value(**obj_pointer) }
    }
}

impl XPCValue<String> for XPCObject {
    fn xpc_value(&self) -> String {
        let XPCObject(obj_pointer, _) = self;
        let cstr = unsafe { CStr::from_ptr(xpc_string_get_string_ptr(**obj_pointer)) };

        cstr.to_string_lossy().to_string()
    }
}

impl XPCValue<bool> for XPCObject {
    fn xpc_value(&self) -> bool {
        let XPCObject(obj_pointer, _) = self;
        unsafe { xpc_bool_get_value(**obj_pointer) }
    }
}

// TODO: can this be read as just uint?
// impl XPCValue<mach_port_t> for XPCObject {
//     fn xpc_value(&self) -> mach_port_t {
//         let XPCObject(obj_pointer, _) = self;
//         unsafe { xpc_mach_send_get_value(**obj_pointer) }
//     }
// }
