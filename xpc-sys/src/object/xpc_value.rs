#![feature(bool_to_option)]

use crate::object::xpc_object::XPCObject;
use crate::object::xpc_type;
use crate::{mach_port_t, xpc_bool_get_value, xpc_int64_get_value, xpc_object_t, xpc_string_get_string_ptr, xpc_uint64_get_value, xpc_type_get_name};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::error::Error;
use crate::object::xpc_type::XPCType;

/// Implement to get data out of xpc_type_t and into
/// a Rust native data type
pub trait TryXPCValue<Out> {
    fn xpc_value(&self) -> Result<Out, XPCValueError>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct XPCValueError(String);

impl Display for XPCValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


fn check_xpc_type(object: &XPCObject, xpc_type: &XPCType) -> Result<(), XPCValueError> {
    let XPCObject(_, obj_type) = object;
    if *obj_type == *xpc_type {
        return Ok(())
    }

    let obj_str = unsafe {
        let XPCType(ptr) = object.xpc_type();
        CStr::from_ptr(xpc_type_get_name(ptr)).to_string_lossy().to_string()
    };

    let req_str = unsafe {
        let XPCType(ptr) = xpc_type;
        CStr::from_ptr(xpc_type_get_name(*ptr)).to_string_lossy().to_string()
    };

    Err(XPCValueError(format!("Cannot get {} as {}", obj_str, req_str)))
}

impl TryXPCValue<i64> for XPCObject {
    fn xpc_value(&self) -> Result<i64, XPCValueError> {
        check_xpc_type(&self, &xpc_type::Int64)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_int64_get_value(**obj_pointer) })
    }
}

impl TryXPCValue<u64> for XPCObject {
    fn xpc_value(&self) -> Result<u64, XPCValueError> {
        check_xpc_type(&self, &xpc_type::UInt64)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_uint64_get_value(**obj_pointer) })
    }
}

impl TryXPCValue<String> for XPCObject {
    fn xpc_value(&self) -> Result<String, XPCValueError> {
        check_xpc_type(&self, &xpc_type::String)?;
        let XPCObject(obj_pointer, _) = self;
        let cstr = unsafe { CStr::from_ptr(xpc_string_get_string_ptr(**obj_pointer)) };

        Ok(cstr.to_string_lossy().to_string())
    }
}

impl TryXPCValue<bool> for XPCObject {
    fn xpc_value(&self) -> Result<bool, XPCValueError> {
        check_xpc_type(&self, &xpc_type::Bool)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_bool_get_value(**obj_pointer) })
    }
}

// TODO: can this be read as just uint?
// impl TryXPCValue<mach_port_t> for XPCObject {
//     fn xpc_value(&self) -> Result<mach_port_t, XPCValueError> {
//         let XPCObject(obj_pointer, _) = self;
//         unsafe { xpc_mach_send_get_value(**obj_pointer) }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::object::xpc_object::XPCObject;
    use crate::object::xpc_value::{XPCValueError, TryXPCValue};

    #[test]
    fn deserialize_as_wrong_type() {
        let an_i64 = XPCObject::from(42 as i64);
        let as_u64: Result<u64, XPCValueError> = an_i64.xpc_value();
        assert_eq!(as_u64.err().unwrap(), XPCValueError("Cannot get int64 as uint64".to_string()));
    }
}