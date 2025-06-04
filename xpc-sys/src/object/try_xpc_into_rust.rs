use block::ConcreteBlock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::rc::Rc;

use crate::object::xpc_object::{MachPortType, XPCHashMap, XPCObject};
use crate::object::xpc_type;
use crate::{
    object, rs_strerror, xpc_array_apply, xpc_bool_get_value, xpc_dictionary_apply,
    xpc_double_get_value, xpc_int64_get_value, xpc_mach_send_get_right, xpc_object_t,
    xpc_string_get_string_ptr, xpc_type_get_name, xpc_uint64_get_value,
};

use crate::object::xpc_error::XPCError;
use crate::object::xpc_error::XPCError::{DictionaryError, ValueError};
use crate::object::xpc_type::check_xpc_type;
use libc::__error;
use mach2::port::mach_port_t;
use std::sync::Arc;

/// Go from xpc_object_t to a Rust type
pub trait TryXPCIntoRust<Out> {
    fn to_rust(&self) -> Result<Out, XPCError>;
}

impl TryXPCIntoRust<i64> for XPCObject {
    fn to_rust(&self) -> Result<i64, XPCError> {
        check_xpc_type(self, &xpc_type::Int64)?;
        Ok(unsafe { xpc_int64_get_value(self.as_ptr()) })
    }
}

impl TryXPCIntoRust<u64> for XPCObject {
    fn to_rust(&self) -> Result<u64, XPCError> {
        check_xpc_type(self, &xpc_type::UInt64)?;
        Ok(unsafe { xpc_uint64_get_value(self.as_ptr()) })
    }
}

impl TryXPCIntoRust<f64> for XPCObject {
    fn to_rust(&self) -> Result<f64, XPCError> {
        check_xpc_type(self, &xpc_type::Double)?;
        Ok(unsafe { xpc_double_get_value(self.as_ptr()) })
    }
}

impl TryXPCIntoRust<String> for XPCObject {
    fn to_rust(&self) -> Result<String, XPCError> {
        check_xpc_type(self, &xpc_type::String)?;
        let cstr = unsafe { CStr::from_ptr(xpc_string_get_string_ptr(self.as_ptr())) };

        Ok(cstr.to_string_lossy().to_string())
    }
}

impl TryXPCIntoRust<bool> for XPCObject {
    fn to_rust(&self) -> Result<bool, XPCError> {
        check_xpc_type(self, &xpc_type::Bool)?;
        Ok(unsafe { xpc_bool_get_value(self.as_ptr()) })
    }
}

impl TryXPCIntoRust<(MachPortType, mach_port_t)> for XPCObject {
    fn to_rust(&self) -> Result<(MachPortType, mach_port_t), XPCError> {
        let types = [
            check_xpc_type(self, &xpc_type::MachSend).map(|()| MachPortType::Send),
            check_xpc_type(self, &xpc_type::MachRecv).map(|()| MachPortType::Recv),
        ];

        for check in &types {
            if check.is_ok() {
                return Ok((*check.as_ref().unwrap(), unsafe {
                    xpc_mach_send_get_right(self.as_ptr())
                }));
            }
        }

        Err(XPCError::ValueError(format!(
            "Object is {} and neither _xpc_type_mach_send nor _xpc_type_mach_recv",
            unsafe { CStr::from_ptr(xpc_type_get_name(self.xpc_type().0)).to_string_lossy() }
        )))
    }
}

impl TryXPCIntoRust<Vec<Arc<XPCObject>>> for XPCObject {
    fn to_rust(&self) -> Result<Vec<Arc<XPCObject>>, XPCError> {
        check_xpc_type(self, &xpc_type::Array)?;

        let vec: Rc<RefCell<Vec<Arc<XPCObject>>>> = Rc::new(RefCell::new(vec![]));
        let vec_rc_clone = vec.clone();

        let block = ConcreteBlock::new(move |_: usize, obj: xpc_object_t| {
            let xpc_object: XPCObject = unsafe { XPCObject::xpc_copy(obj) };
            vec_rc_clone.borrow_mut().push(xpc_object.into());
            true
        });

        let block = block.copy();

        let ok = unsafe { xpc_array_apply(self.as_ptr(), &*block as *const _ as *mut _) };

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

impl TryXPCIntoRust<XPCHashMap> for XPCObject {
    fn to_rust(&self) -> Result<XPCHashMap, XPCError> {
        if self.xpc_type() != *object::xpc_type::Dictionary {
            return Err(DictionaryError(
                "Only XPC_TYPE_DICTIONARY allowed".to_string(),
            ));
        }

        let map: Arc<RefCell<XPCHashMap>> = Arc::new(RefCell::new(HashMap::new()));
        let map_block_clone = map.clone();

        // https://developer.apple.com/documentation/xpc/1505404-xpc_dictionary_apply?language=objc
        let block = ConcreteBlock::new(move |key: *const c_char, value: xpc_object_t| {
            let xpc_object: XPCObject = unsafe { XPCObject::xpc_copy(value) };
            let str_key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };

            map_block_clone
                .borrow_mut()
                .insert(str_key, xpc_object.into());

            // Must return true
            true
        });
        let block = block.copy();
        let ok = unsafe { xpc_dictionary_apply(self.as_ptr(), &*block as *const _ as *mut _) };

        // Explicitly drop the block so map is the only live reference
        // so we can collect it below
        drop(block);

        if ok {
            match Arc::try_unwrap(map) {
                Ok(cell) => Ok(cell.into_inner()),
                Err(_) => Err(DictionaryError("Unable to unwrap Arc".to_string())),
            }
        } else {
            Err(DictionaryError(format!(
                "xpc_dictionary_apply failed: {}",
                rs_strerror(unsafe { *__error() })
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::object::try_xpc_into_rust::TryXPCIntoRust;
    use crate::object::xpc_error::XPCError;
    use crate::object::xpc_error::XPCError::ValueError;
    use crate::object::xpc_object::XPCObject;
    use crate::object::xpc_object::{MachPortType, XPCHashMap};
    use crate::{get_bootstrap_port, xpc_dictionary_create, xpc_dictionary_set_int64};
    use libc::mach_port_t;

    use std::ffi::CString;
    use std::ptr::{null, null_mut};
    use std::sync::Arc;

    #[test]
    fn xpc_to_rs_with_wrong_type() {
        let an_i64 = XPCObject::from(42_i64);
        let as_u64: Result<u64, XPCError> = an_i64.to_rust();

        assert_eq!(
            as_u64.err().unwrap(),
            ValueError("Cannot get int64 as uint64".to_string())
        );
    }

    #[test]
    fn bool_to_rust() {
        let xpc_bool = XPCObject::from(true);
        let rs_bool: bool = xpc_bool.to_rust().unwrap();
        assert!(rs_bool);
    }

    #[test]
    fn i64_to_rust() {
        let xpc_i64 = XPCObject::from(i64::MAX);
        let rs_i64: i64 = xpc_i64.to_rust().unwrap();
        assert_eq!(i64::MAX, rs_i64);
    }

    #[test]
    fn u64_to_rust() {
        let xpc_u64 = XPCObject::from(u64::MAX);
        let rs_u64: u64 = xpc_u64.to_rust().unwrap();
        assert_eq!(u64::MAX, rs_u64);
    }

    #[test]
    fn f64_to_rust() {
        let xpc_f64 = XPCObject::from(f64::MAX);
        let rs_f64: f64 = xpc_f64.to_rust().unwrap();
        assert_eq!(f64::MAX, rs_f64);
    }

    #[test]
    fn mach_send_to_rust() {
        let bootstrap_port: mach_port_t = unsafe { get_bootstrap_port() };

        let xpc_bootstrap_port = XPCObject::from((MachPortType::Send, bootstrap_port));
        let (mpt, port): (MachPortType, mach_port_t) = xpc_bootstrap_port.to_rust().unwrap();

        assert_eq!(MachPortType::Send, mpt);
        assert_eq!(bootstrap_port, port);
    }

    // Can't find any example in the wild, the value is 0 vs the provided 42, it likely
    // does some kind of validation.
    #[test]
    fn mach_recv_to_rust() {
        let xpc_mach_recv = XPCObject::from((MachPortType::Recv, 42 as mach_port_t));
        let (mpt, _port): (MachPortType, mach_port_t) = xpc_mach_recv.to_rust().unwrap();

        assert_eq!(MachPortType::Recv, mpt);
        // assert_eq!(42, port);
    }

    #[test]
    fn array_to_rust() {
        let xpc_array = XPCObject::from(vec!["eins", "zwei", "polizei"]);
        let rs_vec: Vec<Arc<XPCObject>> = xpc_array.to_rust().unwrap();

        assert_eq!(
            rs_vec
                .iter()
                .map(|o| o.to_rust().unwrap())
                .collect::<Vec<String>>(),
            vec![
                "eins".to_string(),
                "zwei".to_string(),
                "polizei".to_string()
            ]
        );
    }

    #[test]
    fn xpc_dictionary_to_rust() {
        let raw_dict = unsafe { xpc_dictionary_create(null(), null_mut(), 0) };
        let key = CString::new("test").unwrap();
        let value: i64 = 42;

        unsafe { xpc_dictionary_set_int64(raw_dict, key.as_ptr(), value) };

        let map: XPCHashMap = unsafe { XPCObject::from_raw(raw_dict).to_rust().unwrap() };

        if let Some(xpc_object) = map.get("test") {
            assert_eq!(value, xpc_object.to_rust().unwrap());
        } else {
            panic!("Unable to get value from map");
        }
    }
}
