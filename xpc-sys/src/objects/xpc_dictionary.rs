use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr::{null, null_mut};
use std::sync::Arc;

use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::DictionaryError;
use crate::objects::xpc_object::XPCObject;
use crate::rs_strerror;
use crate::{objects, xpc_retain};
use crate::{xpc_dictionary_apply, xpc_dictionary_create, xpc_dictionary_set_value, xpc_object_t, errno};

use block::ConcreteBlock;

#[derive(Debug, Clone)]
pub struct XPCDictionary(pub HashMap<String, XPCObject>);

impl XPCDictionary {
    pub fn new() -> Self {
        XPCDictionary(HashMap::new())
    }

    /// Get value from XPCDictionary with support for nesting
    pub fn get<I, S>(&self, items: I) -> Result<XPCObject, XPCError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = items.into_iter();
        let first: S = iter
            .next()
            .ok_or(XPCError::ValueError("Not enough elements".to_string()))?;
        let XPCDictionary(ref hm) = self;

        let first = hm
            .get(first.as_ref())
            .ok_or(XPCError::StandardError)
            .map(|o| o.clone());

        iter.fold(first, |o, k: S| {
            if o.is_err() {
                return o;
            }

            let key = k.as_ref();
            let XPCDictionary(ref inner) = o.unwrap().try_into()?;

            inner
                .get(key)
                .ok_or(XPCError::DictionaryError(format!("Can't get {}", key)))
                .map(|i| i.clone())
        })
    }

    /// Retrieve a dictionary
    pub fn get_as_dictionary<I, S>(&self, items: I) -> Result<XPCDictionary, XPCError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.get(items).and_then(|r| XPCDictionary::try_from(r))
    }
}

impl From<HashMap<String, XPCObject>> for XPCDictionary {
    fn from(dict: HashMap<String, XPCObject>) -> XPCDictionary {
        XPCDictionary(dict)
    }
}

impl TryFrom<&XPCObject> for XPCDictionary {
    type Error = XPCError;

    fn try_from(object: &XPCObject) -> Result<XPCDictionary, XPCError> {
        let XPCObject(_, object_type) = *object;

        if object_type != *objects::xpc_type::Dictionary {
            return Err(DictionaryError(
                "Only XPC_TYPE_DICTIONARY allowed".to_string(),
            ));
        }

        let map: Arc<RefCell<HashMap<String, XPCObject>>> = Arc::new(RefCell::new(HashMap::new()));
        let map_block_clone = map.clone();

        // https://developer.apple.com/documentation/xpc/1505404-xpc_dictionary_apply?language=objc
        let block = ConcreteBlock::new(move |key: *const c_char, value: xpc_object_t| {
            // Prevent xpc_release() collection on block exit
            unsafe { xpc_retain(value) };
            let str_key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };
            map_block_clone.borrow_mut().insert(str_key, value.into());

            // Must return true
            true
        });
        let block = block.copy();
        let ok = unsafe { xpc_dictionary_apply(object.as_ptr(), &*block as *const _ as *mut _) };

        // Explicitly drop the block so map is the only live reference
        // so we can collect it below
        drop(block);

        if ok {
            match Arc::try_unwrap(map) {
                Ok(cell) => Ok(XPCDictionary(cell.into_inner())),
                Err(_) => Err(DictionaryError("Unable to unwrap Arc".to_string())),
            }
        } else {
            Err(DictionaryError(format!("xpc_dictionary_apply failed: {}", rs_strerror(unsafe { errno }))))
        }
    }
}

impl TryFrom<XPCObject> for XPCDictionary {
    type Error = XPCError;

    fn try_from(value: XPCObject) -> Result<XPCDictionary, XPCError> {
        (&value).try_into()
    }
}

impl TryFrom<xpc_object_t> for XPCDictionary {
    type Error = XPCError;

    /// Creates a XPC dictionary from an xpc_object_t pointer. Errors are generally
    /// related to passing in objects other than XPC_TYPE_DICTIONARY
    fn try_from(value: xpc_object_t) -> Result<XPCDictionary, XPCError> {
        let obj: XPCObject = value.into();
        obj.try_into()
    }
}

impl<S> From<HashMap<S, XPCObject>> for XPCObject
where
    S: Into<String>,
{
    /// Creates a XPC dictionary
    ///
    /// Values must be XPCObject but can encapsulate any
    /// valid xpc_object_t
    fn from(message: HashMap<S, XPCObject>) -> Self {
        let dict = unsafe { xpc_dictionary_create(null(), null_mut(), 0) };

        for (k, v) in message {
            unsafe {
                let as_str: String = k.into();
                let cstr = CString::new(as_str).unwrap();
                xpc_dictionary_set_value(dict, cstr.as_ptr(), v.as_ptr());
            }
        }

        dict.into()
    }
}

impl From<&XPCDictionary> for XPCObject {
    fn from(XPCDictionary(map): &XPCDictionary) -> Self {
        map.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::objects::xpc_dictionary::XPCDictionary;
    use crate::objects::xpc_object::XPCObject;
    use crate::traits::xpc_value::TryXPCValue;
    use crate::{xpc_dictionary_create, xpc_dictionary_get_string, xpc_dictionary_set_int64};
    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::ffi::{CStr, CString};
    use std::ptr::{null, null_mut};

    #[test]
    fn raw_to_hashmap() {
        let raw_dict = unsafe { xpc_dictionary_create(null(), null_mut(), 0) };
        let key = CString::new("test").unwrap();
        let value: i64 = 42;

        unsafe { xpc_dictionary_set_int64(raw_dict, key.as_ptr(), value) };

        let XPCDictionary(map) = raw_dict.try_into().unwrap();
        if let Some(xpc_object) = map.get("test") {
            assert_eq!(value, xpc_object.xpc_value().unwrap());
        } else {
            panic!("Unable to get value from map");
        }
    }

    #[test]
    fn hashmap_to_raw() {
        let mut hm: HashMap<&str, XPCObject> = HashMap::new();
        let value = "foo";
        hm.insert("test", XPCObject::from(value));

        let xpc_object = XPCObject::from(hm);
        let cstr = unsafe {
            CStr::from_ptr(xpc_dictionary_get_string(
                xpc_object.as_ptr(),
                CString::new("test").unwrap().as_ptr(),
            ))
        };

        assert_eq!(cstr.to_str().unwrap(), value);
    }
}
