use crate::objects::types::XPCObject;
use crate::{objects, xpc_retain};
use crate::{
    xpc_dictionary_apply, xpc_dictionary_create, xpc_dictionary_set_value, xpc_get_type,
    xpc_object_t, xpc_type_t,
};
use block::ConcreteBlock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};

use std::os::raw::c_char;
use std::ptr::{null, null_mut};
use std::rc::Rc;

#[derive(Debug)]
pub struct XPCDictionaryError(String);

impl Display for XPCDictionaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for XPCDictionaryError {}

pub struct XPCDictionary(pub HashMap<String, XPCObject>);
impl XPCDictionary {
    /// Reify xpc_object_t dictionary as a Rust HashMap
    pub fn new(object: XPCObject) -> Result<XPCDictionary, XPCDictionaryError> {
        if object.get_type() != *objects::types::Dictionary {
            return Err(XPCDictionaryError(
                "Only XPC_TYPE_DICTIONARY allowed".to_string(),
            ));
        }

        let map: Rc<RefCell<HashMap<String, XPCObject>>> = Rc::new(RefCell::new(HashMap::new()));
        let map_rc_clone = map.clone();

        let block = ConcreteBlock::new(move |key: *const c_char, value: xpc_object_t| {
            // Prevent xpc_release() collection on block exit
            unsafe { xpc_retain(value) };

            let str_key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };
            map_rc_clone.borrow_mut().insert(str_key, value.into());
        });
        let block = block.copy();

        let ok = unsafe { xpc_dictionary_apply(object.as_ptr(), &*block as *const _ as *mut _) };

        // Explicitly drop the block so map is the only live reference
        // so we can collect it below
        std::mem::drop(block);

        if ok {
            match Rc::try_unwrap(map) {
                Ok(cell) => Ok(XPCDictionary(cell.into_inner())),
                Err(_) => Err(XPCDictionaryError("Unable to unwrap Rc".to_string())),
            }
        } else {
            Err(XPCDictionaryError(
                "xpc_dictionary_apply failed".to_string(),
            ))
        }
    }
}

impl From<HashMap<String, XPCObject>> for XPCDictionary {
    fn from(dict: HashMap<String, XPCObject>) -> XPCDictionary {
        XPCDictionary(dict)
    }
}

impl TryFrom<xpc_object_t> for XPCDictionary {
    type Error = XPCDictionaryError;

    /// Creates a XPC dictionary from an xpc_object_t pointer. Errors are generally
    /// related to passing in objects other than XPC_TYPE_DICTIONARY
    fn try_from(value: xpc_object_t) -> Result<XPCDictionary, XPCDictionaryError> {
        let obj: XPCObject = value.into();
        XPCDictionary::new(obj)
    }
}

impl From<HashMap<&str, XPCObject>> for XPCObject {
    /// Creates a XPC dictionary
    ///
    /// Values must be XPCObject newtype but can encapsulate any
    /// valid xpc_object_t
    fn from(message: HashMap<&str, XPCObject>) -> Self {
        let dict = unsafe { xpc_dictionary_create(null(), null_mut(), 0) };

        for (k, v) in message {
            unsafe {
                let cstr = CString::new(k);
                xpc_dictionary_set_value(dict, cstr.unwrap().as_ptr(), v.as_ptr());
            }
        }

        dict.into()
    }
}
