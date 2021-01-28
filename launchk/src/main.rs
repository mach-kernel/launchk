/*
basic XPC message example
dstancu@nyu.edu
*/

use cursive::views::{Dialog, TextView};

use xpc_bindgen;
use xpc_bindgen::{xpc_connection_create_mach_service, xpc_connection_t, XPC_CONNECTION_MACH_SERVICE_PRIVILEGED, dispatch_queue_create, xpc_connection_set_event_handler, xpc_object_t, xpc_copy_description, xpc_int64_create, xpc_dictionary_create, xpc_connection_send_message, xpc_string_create, xpc_connection_resume, XPC_CONNECTION_MACH_SERVICE_LISTENER, xpc_bool_create, xpc_connection_create};
use std::ffi::CString;
use std::os::raw::{c_char};
use std::ptr::null_mut;
use block::ConcreteBlock;
use std::ops::Deref;
use cursive::view::AnyView;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::mem;

use std::sync::mpsc;
use futures::channel::mpsc::unbounded;
use futures::SinkExt;
use std::thread::sleep;

static APP_ID: &str = "com.dstancu.launchk";

fn xpc_i64(value: i64) -> Rc<xpc_object_t> {
    unsafe {
        let new_obj = xpc_int64_create(value);
        let rc = Rc::new(new_obj);
        rc
        // let cloned = rc.clone();
        // cloned.to_owned()
    }
}

fn xpc_str(value: &str) -> Rc<xpc_object_t> {
    unsafe {
        let new_obj = xpc_string_create(CString::new(value).unwrap().into_boxed_c_str().as_ptr());
        let rc = Rc::new(new_obj);
        rc
        // let cloned = rc.clone();
        // cloned.to_owned()
    }
}

fn xpc_bool(value: bool) -> Rc<xpc_object_t> {
    unsafe {
        let new_obj = xpc_bool_create(value);
        let rc = Rc::new(new_obj);
        rc
        // let cloned = rc.clone();
        // cloned.to_owned()
    }
}

fn main() {
    let app_id_cstr = CString::new(APP_ID)
        .unwrap()
        .into_boxed_c_str()
        .as_ptr();

    let queue = unsafe {
        dispatch_queue_create(app_id_cstr, null_mut())
    };

    let connection: xpc_connection_t = unsafe {
        xpc_connection_create_mach_service(
            app_id_cstr,
            queue,
            XPC_CONNECTION_MACH_SERVICE_LISTENER as u64
        )
        // xpc_connection_create(app_id_cstr, queue)
    };

    mem::forget(app_id_cstr);

    let (tx, mut rx) = unbounded::<xpc_object_t>();

    let handler = ConcreteBlock::new(move |obj: xpc_object_t| tx.unbounded_send(obj));
    let handler = handler.copy();

    // Register handler
    unsafe {
        xpc_connection_set_event_handler(connection, &*handler as *const _ as *mut _);
        xpc_connection_resume(connection);
    }

    let mut message: HashMap<String, xpc_object_t> = HashMap::new();

    message.insert("type".to_string(), *xpc_i64(7));
    message.insert("handle".to_string(), *xpc_i64(0));
    message.insert("subsystem".to_string(), *xpc_i64(3));
    message.insert("routine".to_string(), *xpc_i64(815));
    message.insert("legacy".to_string(), *xpc_bool(true));
    message.insert("name".to_string(), *xpc_str("com.apple.Spotlight"));

    let mut xpc_dict_keys = Vec::new();
    let mut xpc_dict_values = Vec::new();

    for (k, v) in &message {
        unsafe {
            let desc = CString::from_raw(xpc_copy_description(v.to_owned()));
            println!("Adding {} {}", k, desc.to_string_lossy());
        }

        let key = CString::new(k.deref()).unwrap();
        unsafe { xpc_dict_keys.push(key.as_ptr()) };
        xpc_dict_values.push(v.to_owned());
        mem::forget(key);
    }

    let msg_dict = unsafe {
        xpc_dictionary_create(
            xpc_dict_keys.into_boxed_slice().as_mut_ptr(),
            xpc_dict_values.into_boxed_slice().as_mut_ptr(),
            message.len() as u64
        )
    };

    unsafe {
        let desc = CString::from_raw(xpc_copy_description(msg_dict));
        println!("Sending {}", desc.to_string_lossy());
        xpc_connection_send_message(connection, msg_dict);
    }

    loop {
        if let Ok(Some(recv)) = rx.try_next() {
            unsafe {
                let desc = CString::from_raw(xpc_copy_description(recv));
                println!("Received message! {}", desc.to_str().unwrap());
            }
        } else {
            println!("Polling...nothing here");
            std::thread::sleep(std::time::Duration::new(10, 0));
        }
    }

    // let mut siv = cursive::default();
    //
    // // Creates a dialog with a single "Quit" button
    // siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
    //                      .title("Cursive")
    //                      .button("Quit", |s| s.quit()));
    //
    // // Starts the event loop.
    // siv.run();
}
