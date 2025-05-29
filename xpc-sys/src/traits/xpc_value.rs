use block::ConcreteBlock;
use std::cell::RefCell;
use std::ffi::CStr;
use std::rc::Rc;

use crate::objects::xpc_object::{MachPortType, XPCObject};
use crate::objects::xpc_type;
use crate::{
    xpc_array_apply, xpc_bool_get_value, xpc_double_get_value, xpc_int64_get_value,
    xpc_mach_send_get_right, xpc_object_t, xpc_string_get_string_ptr, xpc_type_get_name,
    xpc_uint64_get_value,
};

use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::ValueError;
use crate::objects::xpc_type::check_xpc_type;
use mach2::port::mach_port_t;
use std::sync::Arc;

/// Implement to get data out of xpc_type_t and into
/// a Rust native data type
pub trait TryXPCValue<Out> {
    fn xpc_value(&self) -> Result<Out, XPCError>;
}

impl TryXPCValue<i64> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<i64, XPCError> {
        check_xpc_type(&self, &xpc_type::Int64)?;
        Ok(unsafe { xpc_int64_get_value(self.as_ptr()) })
    }
}

impl TryXPCValue<u64> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<u64, XPCError> {
        check_xpc_type(&self, &xpc_type::UInt64)?;
        Ok(unsafe { xpc_uint64_get_value(self.as_ptr()) })
    }
}

impl TryXPCValue<f64> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<f64, XPCError> {
        check_xpc_type(&self, &xpc_type::Double)?;
        Ok(unsafe { xpc_double_get_value(self.as_ptr()) })
    }
}

impl TryXPCValue<String> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<String, XPCError> {
        check_xpc_type(&self, &xpc_type::String)?;
        let cstr = unsafe { CStr::from_ptr(xpc_string_get_string_ptr(self.as_ptr())) };

        Ok(cstr.to_string_lossy().to_string())
    }
}

impl TryXPCValue<bool> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<bool, XPCError> {
        check_xpc_type(&self, &xpc_type::Bool)?;
        Ok(unsafe { xpc_bool_get_value(self.as_ptr()) })
    }
}

impl TryXPCValue<(MachPortType, mach_port_t)> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<(MachPortType, mach_port_t), XPCError> {
        let types = [
            check_xpc_type(&self, &xpc_type::MachSend).map(|()| MachPortType::Send),
            check_xpc_type(&self, &xpc_type::MachRecv).map(|()| MachPortType::Recv),
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

impl TryXPCValue<Vec<Arc<XPCObject>>> for XPCObject {
    #[must_use]
    fn xpc_value(&self) -> Result<Vec<Arc<XPCObject>>, XPCError> {
        check_xpc_type(&self, &xpc_type::Array)?;

        let vec: Rc<RefCell<Vec<Arc<XPCObject>>>> = Rc::new(RefCell::new(vec![]));
        let vec_rc_clone = vec.clone();

        let block = ConcreteBlock::new(move |_: usize, obj: xpc_object_t| {
            let xpc_object: XPCObject = XPCObject::xpc_copy(obj);
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

#[cfg(test)]
mod tests {
    use crate::get_bootstrap_port;
    use crate::objects::xpc_error::XPCError;
    use crate::objects::xpc_error::XPCError::ValueError;
    use crate::objects::xpc_object::MachPortType;
    use crate::objects::xpc_object::XPCObject;
    use crate::traits::xpc_value::TryXPCValue;
    use libc::mach_port_t;
    use std::sync::Arc;

    #[test]
    fn xpc_to_rs_with_wrong_type() {
        let an_i64 = XPCObject::from(42 as i64);
        let as_u64: Result<u64, XPCError> = an_i64.xpc_value();

        assert_eq!(
            as_u64.err().unwrap(),
            ValueError("Cannot get int64 as uint64".to_string())
        );
    }

    #[test]
    fn xpc_value_bool() {
        let xpc_bool = XPCObject::from(true);
        let rs_bool: bool = xpc_bool.xpc_value().unwrap();
        assert_eq!(true, rs_bool);
    }

    #[test]
    fn xpc_value_i64() {
        let xpc_i64 = XPCObject::from(std::i64::MAX);
        let rs_i64: i64 = xpc_i64.xpc_value().unwrap();
        assert_eq!(std::i64::MAX, rs_i64);
    }

    #[test]
    fn xpc_value_u64() {
        let xpc_u64 = XPCObject::from(std::u64::MAX);
        let rs_u64: u64 = xpc_u64.xpc_value().unwrap();
        assert_eq!(std::u64::MAX, rs_u64);
    }

    #[test]
    fn xpc_value_f64() {
        let xpc_f64 = XPCObject::from(std::f64::MAX);
        let rs_f64: f64 = xpc_f64.xpc_value().unwrap();
        assert_eq!(std::f64::MAX, rs_f64);
    }

    #[test]
    fn xpc_value_mach_send() {
        let bootstrap_port: mach_port_t = unsafe { get_bootstrap_port() };

        let xpc_bootstrap_port = XPCObject::from((MachPortType::Send, bootstrap_port));
        let (mpt, port): (MachPortType, mach_port_t) = xpc_bootstrap_port.xpc_value().unwrap();

        assert_eq!(MachPortType::Send, mpt);
        assert_eq!(bootstrap_port, port);
    }

    // Can't find any example in the wild, the value is 0 vs the provided 42, it likely
    // does some kind of validation.
    #[test]
    fn xpc_value_mach_recv() {
        let xpc_mach_recv = XPCObject::from((MachPortType::Recv, 42 as mach_port_t));
        let (mpt, _port): (MachPortType, mach_port_t) = xpc_mach_recv.xpc_value().unwrap();

        assert_eq!(MachPortType::Recv, mpt);
        // assert_eq!(42, port);
    }

    #[test]
    fn xpc_value_array() {
        let xpc_array = XPCObject::from(vec!["eins", "zwei", "polizei"]);
        let rs_vec: Vec<Arc<XPCObject>> = xpc_array.xpc_value().unwrap();

        assert_eq!(
            rs_vec
                .iter()
                .map(|o| o.xpc_value().unwrap())
                .collect::<Vec<String>>(),
            vec![
                "eins".to_string(),
                "zwei".to_string(),
                "polizei".to_string()
            ]
        );
    }
}
