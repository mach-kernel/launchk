# xpc-sys

[![Rust](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml) ![crates.io](https://img.shields.io/crates/v/xpc-sys.svg)

Various utilities for conveniently dealing with XPC in Rust.

- [Object lifecycle](#object-lifecycle)
- [XPC Dictionary](#xpc-dictionary)
- [XPC Array](#xpc-array)
- [XPC Shmem](#xpc-shmem)
- [Pipe Routine API](#api)

#### Getting Started

Conversions to/from Rust/XPC objects uses the [xpc.h functions documented on Apple Developer](https://developer.apple.com/documentation/xpc/xpc_services_xpc_h?language=objc) using the `From` trait.

| Rust                                   | XPC                        |
|----------------------------------------|----------------------------|
| i64/i32                                | _xpc_type_int64            |
| u64/u32                                | _xpc_type_uint64           |
| f64                                    | _xpc_type_double           |
| bool                                   | _xpc_bool_true/false       |
| Into<String>                           | _xpc_type_string           |
| HashMap<Into<String>, Into<XPCObject>> | _xpc_type_dictionary       |
| Vec<Into<XPCObject>>                   | _xpc_type_array            |
| std::os::unix::prelude::RawFd          | _xpc_type_fd               |
| (MachPortType::Send, mach_port_t)      | _xpc_type_mach_send        |
| (MachPortType::Recv, mach_port_t)      | _xpc_type_mach_recv        |
| XPCShmem                               | _xpc_type_shmem            |

Make XPC objects for anything with `From<T>`. `XPCShmem` and file descriptors have their own constructors:
```rust
use xpc_sys::api::dict_builder::DictBuilder;

let fd = unsafe { XPCObject::from_raw_fd(42) };

let shmem = XPCShmem::allocate_task_self(
  1_000_000,
  MAP_SHARED,
)?;

// pub type XPCHashMap = HashMap<String, Arc<XPCObject>>
let dict: XPCHashMap = HashMap::new()
    .entry("fd", fd)
    .entry("shmem", &shmem.xpc_object);
```

Go from an XPC object to value via `to_rust()` from the `TryXPCIntoRust` trait. Object types are checked with `xpc_get_type()` to yield a clear error if trying to read as the wrong type:
```rust
#[test]
fn deserialize_as_wrong_type() {
    let an_i64: XPCObject = XPCObject::from(42 as i64);
    let as_u64: Result<u64, XPCError> = an_i64.to_rust();
    assert_eq!(
        as_u64.err().unwrap(),
        XPCValueError("Cannot get int64 as uint64".to_string())
    );
}
```

[Top](#xpc-sys)

#### Object lifecycle

XPCObject wraps a `xpc_object_t`:

```rust
pub struct XPCObject(pub xpc_object_t, pub XPCType);
```

When it is dropped, [`xpc_release`](https://developer.apple.com/documentation/xpc/1505851-xpc_release) is called.

[Top](#xpc-sys)

#### XPC Dictionary

Go from a `HashMap` to `xpc_object_t`:

```rust
use xpc_sys::api::dict_builder::DictBuilder;

// pub type XPCHashMap = HashMap<String, Arc<XPCObject>>
let dict: XPCHashMap = HashMap::new()
    .entry("type", 1u64);
    .entry("handle", 0u64);
    .entry("subsystem", 3u64);
    .entry("routine", 815u64);
    .entry("legacy", true);

let xpc_object: XPCObject = dict.into()
let ptr: xpc_object_t = xpc_object.as_ptr();
```

Go from `XPCObject` back to `HashMap`:

```rust
let xpc_object: XPCObject = unsafe { XPCObject::from_raw(some_pointer) };

// Error if something is wrong during conversion (e.g. the pointer is not a XPC dictionary)
match xpc_object.to_rust() {
  Ok(dict) => dict.get("some_key"),
  Err(e) => ...
}
```

[Top](#xpc-sys)

#### XPC Array

An XPC array can be made from either `Vec<XPCObject>` or `Vec<Into<XPCObject>>`:

```rust
let xpc_array = XPCObject::from(vec![XPCObject::from("eins"), XPCObject::from("zwei"), XPCObject::from("polizei")]);
let xpc_array = XPCObject::from(vec!["eins", "zwei", "polizei"]);
```

Go back to `Vec` using `xpc_value`:

```rust
let rs_vec: Vec<XPCObject> = xpc_array.xpc_value().unwrap();
```

[Top](#xpc-sys)

#### XPC Shmem

Make XPC shared memory objects by providing a size and vm_allocate/mmap flags. [`vm_allocate`](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/ManagingMemory/Articles/MemoryAlloc.html) is used to create the memory region, and `vm_deallocate` when `XPCShmem` is dropped.

```rust
let shmem = XPCShmem::new_task_self(
    0x1400000,
    i32::try_from(MAP_SHARED).expect("Must conv flags"),
)?;

// Use _xpc_type_shmem value in XPC Dictionary
let response = HashMap::new()
    .entry("shmem", &shmem)
    .pipe_routine_with_error_handling()?;
```

To work with the shmem region, use [`slice_from_raw_parts`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html):

```rust
let bytes: &[u8] = unsafe {
    &*slice_from_raw_parts(shmem.region as *mut u8, size)
};

// Make a string from bytes in the shmem
let mut hey_look_a_string = String::new();
bytes.read_to_string(buf);
```

[Top](#xpc-sys)

#### API

The following XPC functions have Rust friendly wrappers, all of which return `Result<XPCObject, XPCError>`:

| Function                    | Rust API                                   |
|-----------------------------|--------------------------------------------|
| xpc_pipe_routine            | api::pipe_routine::pipe_routine            |
| xpc_pipe_routine_with_flags | api::pipe_routine::pipe_routine_with_flags |
| _xpc_pipe_interface_routine | api::pipe_routine::pipe_interface_routine  |

If desired, errors in the XPC reply can be handled by chaining `api::handle_reply_dict_errors` onto the pipe routine call.

This is an example of sending `launchctl bootout` via the XPC bootstrap pipe:

```rust
let dict = HashMap::new()
    .entry("name", label_string)
    .entry("no-einprogress", true)
    // Current user UID
    .entry("handle", 501u64)
    // Domain
    .entry("type", 8u64);

let reply: XPCHashMap = pipe_interface_routine(
    // Some(xpc_pipe_t) or fall back to `get_xpc_bootstrap_pipe()`
    None,
    // routine
    801,
    dict,
    // flags (or fall back to 0)
    None
)
    // Check for errors in response XPC dictionary (if desired)
    .and_then(handle_reply_dict_errors)
    // Convert reply to a Rust hash map
    .and_then(|o| o.to_rust())
```

[Top](#xpc-sys)

### Credits

A big thanks to these open source projects and general resources:

- [block](https://crates.io/crates/block) Obj-C block support, necessary for any XPC function taking `xpc_*_applier_t`  
- [Cursive](https://github.com/gyscos/cursive)
- [tokio](https://github.com/tokio-rs/tokio)
- [plist](https://crates.io/crates/plist)
- [notify](https://docs.rs/notify/4.0.16/notify/)
- [bitflags](https://docs.rs/bitflags/1.2.1/bitflags/)  
- [libc](https://crates.io/crates/libc)
- [lazy_static](https://crates.io/crates/lazy_static)
- [xcrun](https://crates.io/crates/xcrun)
- [Apple Developer XPC services](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingXPCServices.html)  
- [Apple Developer XPC API reference](https://developer.apple.com/documentation/xpc?language=objc)  
- [MOXIL / launjctl](http://newosxbook.com/articles/jlaunchctl.html)  
- [geosnow - A Long Evening With macOS' sandbox](https://geosn0w.github.io/A-Long-Evening-With-macOS%27s-Sandbox/)  
- [Bits of launchd - @5aelo](https://saelo.github.io/presentations/bits_of_launchd.pdf)  
- [Audit tokens explained (e.g. ASID)](https://knight.sc/reverse%20engineering/2020/03/20/audit-tokens-explained.html)  
- [objc.io XPC guide](https://www.objc.io/issues/14-mac/xpc/)
- [Fortinet XPC RE article](https://www.fortinet.com/blog/threat-research/a-look-into-xpc-internals--reverse-engineering-the-xpc-objects)
- The various source links found in comments, from Chrome's sandbox and other headers with definitions for private API functions.
