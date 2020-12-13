#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_void, c_double};
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// #[repr(C)]
// union _launch_data_ptrs {
//     _array: *mut launch_data_t,
//     string: *mut char,
//     opaque: *mut c_void,
//     __junk: i64,
// }
//
// #[repr(C)]
// union _launch_data_sizes {
//     _array_cnt: u64,
//     string_len: u64,
//     opaque_size: u64,
// }
//
// #[repr(C)]
// struct _launch_data_data {
//     ptrs: _launch_data_ptrs,
//     sizes: _launch_data_sizes,
// }
//
// #[repr(C)]
// union _launch_data_body {
//     data: _launch_data_data,
//     fd: i64,
//     mp: u64,
//     err: u64,
//     number: i64,
//     boolean: u64,
//     float_num: c_double,
// }

#[repr(C)]
struct _launch_data {
    ld_type: _launch_data_type_t,
    body: _launch_data_body,
}

pub fn get_agent(label: &str) -> Vec<String> {
    let ld: launch_data_t = _launch_data {
        ld_type: launch_data_type_t_LAUNCH_DATA_DICTIONARY,
        body: _launch_data_body {},
    };

    return vec![];
}
