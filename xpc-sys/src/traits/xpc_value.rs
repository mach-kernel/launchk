use block::ConcreteBlock;
use std::cell::RefCell;
use std::ffi::CStr;
use std::rc::Rc;

use crate::objects::xpc_object::{MachPortType, XPCObject};
use crate::objects::xpc_type;
use crate::{
    mach_port_t, size_t, xpc_array_apply, xpc_bool_get_value, xpc_double_get_value,
    xpc_int64_get_value, xpc_mach_send_get_right, xpc_object_t, xpc_retain,
    xpc_string_get_string_ptr, xpc_type_get_name, xpc_uint64_get_value,
};

use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::ValueError;
use crate::objects::xpc_type::check_xpc_type;

/// Implement to get data out of xpc_type_t and into
/// a Rust native data type
pub trait TryXPCValue<Out> {
    fn xpc_value(&self) -> Result<Out, XPCError>;
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

impl TryXPCValue<f64> for XPCObject {
    fn xpc_value(&self) -> Result<f64, XPCError> {
        check_xpc_type(&self, &xpc_type::Double)?;
        let XPCObject(obj_pointer, _) = self;
        Ok(unsafe { xpc_double_get_value(**obj_pointer) })
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

impl TryXPCValue<(MachPortType, mach_port_t)> for XPCObject {
    fn xpc_value(&self) -> Result<(MachPortType, mach_port_t), XPCError> {
        let XPCObject(obj_pointer, xpc_type) = self;

        let types = [
            check_xpc_type(&self, &xpc_type::MachSend).map(|()| MachPortType::Send),
            check_xpc_type(&self, &xpc_type::MachRecv).map(|()| MachPortType::Recv),
        ];

        for check in types.iter() {
            if check.is_ok() {
                return Ok((*check.as_ref().unwrap(), unsafe {
                    xpc_mach_send_get_right(**obj_pointer)
                }));
            }
        }

        Err(XPCError::ValueError(format!(
            "Object is {} and neither _xpc_type_mach_send nor _xpc_type_mach_recv",
            unsafe { CStr::from_ptr(xpc_type_get_name(xpc_type.0)).to_string_lossy() }
        )))
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

        let ok = unsafe { xpc_array_apply(**arc, &*block as *const _ as *mut _) };

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
    use crate::mach_port_t;
    use crate::objects::xpc_error::XPCError;
    use crate::objects::xpc_error::XPCError::ValueError;
    use crate::objects::xpc_object::MachPortType;
    use crate::objects::xpc_object::XPCObject;
    use crate::traits::xpc_value::TryXPCValue;

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
    fn xpc_value_vec() {
        let xpc_array = XPCObject::from(vec![XPCObject::from("ohai")]);
        let vec: Vec<XPCObject> = xpc_array.xpc_value().unwrap();
        let ohai: String = vec.get(0).unwrap().xpc_value().unwrap();
        assert_eq!(ohai, "ohai");
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
        let xpc_bootstrap_port =
            XPCObject::from((MachPortType::Send, get_bootstrap_port() as mach_port_t));
        let (mpt, port): (MachPortType, mach_port_t) = xpc_bootstrap_port.xpc_value().unwrap();

        assert_eq!(MachPortType::Send, mpt);
        assert_eq!(get_bootstrap_port(), port);
    }

    // Can't find any example in the wild, the value is 0 vs the provided 42, it likely
    // does some kind of validation.
    //
    // #[test]
    // fn xpc_value_mach_recv() {
    //     let xpc_mach_recv = XPCObject::from((MachPortType::Recv, 42 as mach_port_t));
    //     let (mpt, port): (MachPortType, mach_port_t) = xpc_mach_recv.xpc_value().unwrap();
    //
    //     assert_eq!(MachPortType::Recv, mpt);
    //     assert_eq!(42, port);
    // }

    #[test]
    fn xpc_value_array() {
        let xpc_array = XPCObject::from(vec!["eins", "zwei", "polizei"]);
        let rs_vec: Vec<XPCObject> = xpc_array.xpc_value().unwrap();

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
