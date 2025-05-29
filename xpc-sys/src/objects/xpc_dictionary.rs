use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr::{null, null_mut};
use std::sync::Arc;

use crate::objects;
use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::DictionaryError;
use crate::objects::xpc_object::XPCObject;
use crate::rs_strerror;
use crate::{
    xpc_dictionary_apply, xpc_dictionary_create, xpc_dictionary_set_value, xpc_object_t,
};

use block::ConcreteBlock;
use libc::__error;

/// A wrapper around Rust HashMap<String, Arc<XPCObject>> that can
/// be Into<XPCObject>
#[derive(Debug, Clone)]
pub struct XPCDictionary(pub HashMap<String, Arc<XPCObject>>);

impl XPCDictionary {
    pub fn new() -> Self {
        XPCDictionary(HashMap::new())
    }

    /// Get value from XPCDictionary with support for nesting
    #[must_use]
    pub fn get<I, S>(&self, items: I) -> Result<Arc<XPCObject>, XPCError>
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
            .ok_or(XPCError::ValueError("Key missing".to_string()))
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
    #[must_use]
    pub fn get_as_dictionary<I, S>(&self, items: I) -> Result<XPCDictionary, XPCError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.get(items).and_then(|r| XPCDictionary::try_from(r))
    }
}

impl From<HashMap<String, Arc<XPCObject>>> for XPCDictionary {
    fn from(dict: HashMap<String, Arc<XPCObject>>) -> XPCDictionary {
        XPCDictionary(dict)
    }
}

impl TryFrom<&XPCObject> for XPCDictionary {
    type Error = XPCError;

    /// Copy data from XPC dictionary into a Rust HashMap
    #[must_use]
    fn try_from(object: &XPCObject) -> Result<XPCDictionary, XPCError> {
        if object.xpc_type() != *objects::xpc_type::Dictionary {
            return Err(DictionaryError(
                "Only XPC_TYPE_DICTIONARY allowed".to_string(),
            ));
        }

        let map: Arc<RefCell<HashMap<String, Arc<XPCObject>>>> =
            Arc::new(RefCell::new(HashMap::new()));
        let map_block_clone = map.clone();

        // https://developer.apple.com/documentation/xpc/1505404-xpc_dictionary_apply?language=objc
        let block = ConcreteBlock::new(move |key: *const c_char, value: xpc_object_t| {
            let xpc_object: XPCObject = XPCObject::xpc_copy(value);
            let str_key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };

            map_block_clone
                .borrow_mut()
                .insert(str_key, xpc_object.into());

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
            Err(DictionaryError(format!(
                "xpc_dictionary_apply failed: {}",
                rs_strerror(unsafe { *__error() })
            )))
        }
    }
}

impl TryFrom<Arc<XPCObject>> for XPCDictionary {
    type Error = XPCError;

    #[must_use]
    fn try_from(value: Arc<XPCObject>) -> Result<XPCDictionary, XPCError> {
        (&*value).try_into()
    }
}

impl TryFrom<XPCObject> for XPCDictionary {
    type Error = XPCError;

    #[must_use]
    fn try_from(value: XPCObject) -> Result<XPCDictionary, XPCError> {
        (&value).try_into()
    }
}

impl TryFrom<xpc_object_t> for XPCDictionary {
    type Error = XPCError;

    /// Creates a XPC dictionary from an xpc_object_t pointer. Errors are generally
    /// related to passing in objects other than XPC_TYPE_DICTIONARY
    #[must_use]
    fn try_from(value: xpc_object_t) -> Result<XPCDictionary, XPCError> {
        let obj: XPCObject = unsafe { XPCObject::from_raw(value) };
        obj.try_into()
    }
}

impl<S> From<HashMap<S, Arc<XPCObject>>> for XPCObject
where
    S: Into<String>,
{
    /// Creates a XPC dictionary
    ///
    /// Values must be Arc<XPCObject> but can encapsulate any
    /// valid xpc_object_t
    fn from(message: HashMap<S, Arc<XPCObject>>) -> Self {
        let dict = unsafe { xpc_dictionary_create(null(), null_mut(), 0) };

        log::trace!("XPCDictionary to object {:p}", dict);

        for (k, v) in message {
            unsafe {
                let as_str: String = k.into();
                let cstr = CString::new(as_str).unwrap();
                log::trace!("Dictionary {:p} add {:?}: {:p}", dict, cstr, v.as_ptr());
                xpc_dictionary_set_value(dict, cstr.as_ptr(), v.as_ptr());
            }
        }

        unsafe { XPCObject::from_raw(dict) }
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
    use std::sync::Arc;

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
        let mut hm: HashMap<&str, Arc<XPCObject>> = HashMap::new();
        let value = "foo";
        hm.insert("test", XPCObject::from(value).into());

        let xpc_object = XPCObject::from(hm);
        let key = CString::new("test").unwrap();

        let cstr =
            unsafe { CStr::from_ptr(xpc_dictionary_get_string(xpc_object.as_ptr(), key.as_ptr())) };

        assert_eq!(cstr.to_str().unwrap(), value);
    }
}
