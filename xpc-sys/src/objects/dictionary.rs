use crate::objects;
use crate::objects::types::{XPCObject, XPCType};
use crate::{
    xpc_dictionary_apply, xpc_dictionary_create, xpc_dictionary_set_value, xpc_get_type,
    xpc_object_t, xpc_type_t,
};
use block::ConcreteBlock;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
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

pub struct XPCDictionary(HashMap<String, XPCObject>);
impl XPCDictionary {
    /// Reify xpc_object_t dictionary as a Rust HashMap
    pub fn new(object: xpc_object_t) -> Result<XPCDictionary, XPCDictionaryError> {
        let object_type: XPCType = unsafe { XPCType(xpc_get_type(object)) };
        if object_type != *objects::types::Dictionary {
            return Err(XPCDictionaryError(
                "Only XPC_TYPE_DICTIONARY allowed".to_string(),
            ));
        }

        let map: Rc<RefCell<HashMap<String, XPCObject>>> = Rc::new(RefCell::new(HashMap::new()));
        let map_rc_clone = map.clone();

        let block = ConcreteBlock::new(move |key: *const c_char, value: xpc_object_t| {
            let str_key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };
            map_rc_clone.borrow_mut().insert(str_key, XPCObject(value));
        });
        let block = block.copy();

        let ok = unsafe { xpc_dictionary_apply(object, &block as *const _ as *mut _) };

        if ok {
            let mut hm: HashMap<String, XPCObject> = HashMap::new();
            for (k, v) in map.borrow().deref() {
                hm.insert(k.clone(), XPCObject(v.0));
            }
            Ok(XPCDictionary(hm))
        } else {
            Err(XPCDictionaryError(
                "xpc_dictionary_apply failed".to_string(),
            ))
        }
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
                xpc_dictionary_set_value(dict, cstr.unwrap().as_ptr(), v.0);
            }
        }

        XPCObject(dict)
    }
}
