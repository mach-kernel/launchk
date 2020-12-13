#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::os::raw::{c_void, c_double};
use std::ffi::CString;

#[repr(C)]
#[derive(Copy, Clone)]
union _launch_data_ptrs {
    _array: *mut launch_data_t,
    string: *mut char,
    opaque: *mut c_void,
    __junk: i64,
}

#[repr(C)]
#[derive(Copy, Clone)]
union _launch_data_sizes {
    _array_cnt: u64,
    string_len: u64,
    opaque_size: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct _launch_data_data {
    ptrs: _launch_data_ptrs,
    sizes: _launch_data_sizes,
}

#[repr(C)]
#[derive(Copy, Clone)]
union _launch_data_body {
    data: _launch_data_data,
    fd: i64,
    mp: u64,
    err: u64,
    number: i64,
    boolean: u64,
    float_num: c_double,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct _launch_data {
    ld_type: launch_data_type_t,
    body: _launch_data_body,
}

impl Default for _launch_data {
    fn default() -> _launch_data {
        let empty: usize = 0;
        _launch_data {
            ld_type: launch_data_type_t_LAUNCH_DATA_OPAQUE,
            body: _launch_data_body {
                data: _launch_data_data {
                    ptrs: _launch_data_ptrs {
                        _array: empty as *mut launch_data_t,
                    },
                    sizes: _launch_data_sizes {
                        _array_cnt: 0,
                    }
                }
            },
        }
    }
}



pub fn get_agent(label: &str) -> Vec<String> {
    let mut ld = _launch_data::default();

    let label_cstr = CString::new(label).unwrap();
    let getjob_cstr = CString::new("GetJob").unwrap();

    // libxpc api misuse??
    unsafe {
        launch_data_dict_insert(
            &mut ld,
            launch_data_new_string(label_cstr.as_ptr()),
            getjob_cstr.as_ptr()
        );
    }

    let resp: *mut _launch_data = unsafe { launch_msg(&mut ld) };
    unsafe { println!("Launch msg response type {}", (*resp).ld_type) };

    return vec![];
}
