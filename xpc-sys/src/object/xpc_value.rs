use crate::object::xpc_type::XPCType;
use crate::object::xpc_object::XPCObject;
use crate::{xpc_int64_get_value, xpc_object_t};

pub trait XPCValue<T> {
    fn xpc_value(&self, obj: xpc_object_t) -> T;
}

impl XPCValue<i64> for XPCType {
    fn xpc_value(&self, obj: xpc_object_t) -> i64 {
        unsafe { xpc_int64_get_value(obj) }
    }
}