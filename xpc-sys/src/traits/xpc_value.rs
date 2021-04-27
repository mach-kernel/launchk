use std::ffi::CStr;

use crate::objects::xpc_object::XPCObject;
use crate::objects::xpc_type;
use crate::{xpc_bool_get_value, xpc_int64_get_value, xpc_string_get_string_ptr, xpc_type_get_name, xpc_uint64_get_value, xpc_object_t, xpc_retain, xpc_array_apply, size_t};

use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::ValueError;
use crate::objects::xpc_type::XPCType;
use block::ConcreteBlock;
use std::rc::Rc;
use std::cell::RefCell;

/// Implement to get data out of xpc_type_t and into
/// a Rust native data type
pub trait TryXPCValue<Out> {
    fn xpc_value(&self) -> Result<Out, XPCError>;
}

fn check_xpc_type(object: &XPCObject, xpc_type: &XPCType) -> Result<(), XPCError> {
    let XPCObject(_, obj_type) = object;
    if *obj_type == *xpc_type {
        return Ok(());
    }

    let obj_str = unsafe {
        let XPCType(ptr) = object.xpc_type();
        CStr::from_ptr(xpc_type_get_name(ptr))
            .to_string_lossy()
            .to_string()
    };

    let req_str = unsafe {
        let XPCType(ptr) = xpc_type;
        CStr::from_ptr(xpc_type_get_name(*ptr))
            .to_string_lossy()
            .to_string()
    };

    Err(ValueError(format!("Cannot get {} as {}", obj_str, req_str)))
}

impl TryXPCValue<i64> for XPCObject {
    fn xpc_value(&self) -> Result<i64, XPCError> {
        check_xpc_type(&self, &xpc_type::Int64)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_int64_get_value(**obj_pointer) })
    }
}

impl TryXPCValue<u64> for XPCObject {
    fn xpc_value(&self) -> Result<u64, XPCError> {
        check_xpc_type(&self, &xpc_type::UInt64)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_uint64_get_value(**obj_pointer) })
    }
}

impl TryXPCValue<String> for XPCObject {
    fn xpc_value(&self) -> Result<String, XPCError> {
        check_xpc_type(&self, &xpc_type::String)?;
        let XPCObject(obj_pointer, _) = self;
        let cstr = unsafe { CStr::from_ptr(xpc_string_get_string_ptr(**obj_pointer)) };

        Ok(cstr.to_string_lossy().to_string())
    }
}

impl TryXPCValue<bool> for XPCObject {
    fn xpc_value(&self) -> Result<bool, XPCError> {
        check_xpc_type(&self, &xpc_type::Bool)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_bool_get_value(**obj_pointer) })
    }
}

impl TryXPCValue<Vec<XPCObject>> for XPCObject {
    fn xpc_value(&self) -> Result<Vec<XPCObject>, XPCError> {
        check_xpc_type(&self, &xpc_type::Array)?;
        let XPCObject(arc, _) = self;

        let vec: Rc<RefCell<Vec<XPCObject>>> = Rc::new(RefCell::new(vec![]));
        let vec_rc_clone = vec.clone();

        let block = ConcreteBlock::new(move |_: size_t, obj: xpc_object_t| {
            unsafe { xpc_retain(obj) };
            vec_rc_clone.borrow_mut().push(obj.into());
        });

        let block = block.copy();

        let ok = unsafe {
            xpc_array_apply(**arc, &*block as *const _ as *mut _)
        };

        drop(block);

        if ok {
            match Rc::try_unwrap(vec) {
                Ok(cell) => Ok(cell.into_inner()),
                Err(_) => Err(ValueError("Unable to unwrap Rc".to_string())),
            }
        } else {
            Err(ValueError("xpc_dictionary_apply failed".to_string()))
        }
    }
}

// TODO: can this be read as just uint?
// impl TryXPCValue<mach_port_t> for XPCObject {
//     fn xpc_value(&self) -> Result<mach_port_t, XPCError> {
//         let XPCObject(obj_pointer, _) = self;
//         unsafe { xpc_mach_send_get_value(**obj_pointer) }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::objects::xpc_error::XPCError;
    use crate::objects::xpc_error::XPCError::ValueError;
    use crate::objects::xpc_object::XPCObject;
    use crate::traits::xpc_value::TryXPCValue;

    #[test]
    fn deserialize_as_wrong_type() {
        let an_i64 = XPCObject::from(42 as i64);
        let as_u64: Result<u64, XPCError> = an_i64.xpc_value();

        assert_eq!(
            as_u64.err().unwrap(),
            ValueError("Cannot get int64 as uint64".to_string())
        );
    }

    #[test]
    fn xpc_value_vec() {
        let xpc_array = XPCObject::from(vec![XPCObject::from("ohai")]);
        let vec: Vec<XPCObject> = xpc_array.xpc_value().unwrap();
        let ohai: String = vec.get(0).unwrap().xpc_value().unwrap();
        assert_eq!(ohai, "ohai");
    }
}
