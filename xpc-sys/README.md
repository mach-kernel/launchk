# xpc-sys

[![Rust](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml) ![crates.io](https://img.shields.io/crates/v/xpc-sys.svg)

Various utilities for conveniently dealing with XPC in Rust.

- [Object lifecycle](#object-lifecycle)
- [QueryBuilder](#query-builder)
- [XPC Dictionary](#xpc-dictionary)
- [XPC Array](#xpc-array)
- [XPC Shmem](#xpc-shmem)

#### Getting Started

Conversions to/from Rust/XPC objects uses the [xpc.h functions documented on Apple Developer](https://developer.apple.com/documentation/xpc/xpc_services_xpc_h?language=objc) using the `From` trait. Complex types such as arrays and shared memory objects described in greater detail below.

| Rust                                   | XPC                        |
|----------------------------------------|----------------------------|
| i64                                    | _xpc_type_int64            |
| u64                                    | _xpc_type_uint64           |
| f64                                    | _xpc_type_double           |
| bool                                   | _xpc_bool_true/false       |
| Into<String>                           | _xpc_type_string           |
| HashMap<Into<String>, Into<XPCObject>> | _xpc_type_dictionary       |
| Vec<Into<XPCObject>>                   | _xpc_type_array            |
| std::os::unix::prelude::RawFd          | _xpc_type_fd               |
| (MachPortType::Send, mach_port_t)      | _xpc_type_mach_send        |
| (MachPortType::Recv, mach_port_t)      | _xpc_type_mach_recv        |
| XPCShmem                               | _xpc_type_shmem            |

Make XPC objects for anything with `From<T>`. Make sure to use the correct type for file descriptors and Mach ports:
```rust
let mut message: HashMap<&str, XPCObject> = HashMap::new();

message.insert(
    "domain-port",
    XPCObject::from((MachPortType::Send, unsafe {
        get_bootstrap_port() as mach_port_t
    })),
);
```

Go from an XPC object to value via the `TryXPCValue` trait. It checks your object's type via `xpc_get_type()` and yields a clear error if you're using the wrong type:
```rust
#[test]
fn deserialize_as_wrong_type() {
    let an_i64: XPCObject = XPCObject::from(42 as i64);
    let as_u64: Result<u64, XPCError> = an_i64.xpc_value();
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

#### QueryBuilder

While we can go from `HashMap<&str, XPCObject>` to `XPCObject`, it can be a little verbose. A `QueryBuilder` trait exposes some builder methods to make building an XPC dictionary a little easier (without all of the `into()`s, and some additional error checking).

To write the query for `launchctl list`:

```rust
    let LIST_SERVICES: XPCDictionary = XPCDictionary::new()
        // "list com.apple.Spotlight" (if specified)
        // .entry("name", "com.apple.Spotlight");
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("routine", 815 as u64)
        .entry("legacy", true);

    let reply: Result<XPCDictionary, XPCError> = XPCDictionary::new()
        // LIST_SERVICES is a proto 
        .extend(&LIST_SERVICES)
        // Specify the domain type, or fall back on requester domain
        .with_domain_type_or_default(Some(domain_type))
        .entry_if_present("name", name)
        .pipe_routine_with_error_handling();
```

In addition to checking `errno` is 0, `pipe_routine_with_error_handling` also looks for possible `error`  and `errors` keys in the response dictionary and provides an `Err()` with `xpc_strerror` contents.

[Top](#xpc-sys)

#### XPC Dictionary

Go from a `HashMap` to `xpc_object_t` with the `XPCObject` type:

```rust
let mut message: HashMap<&str, XPCObject> = HashMap::new();
message.insert("type", XPCObject::from(1 as u64));
message.insert("handle", XPCObject::from(0 as u64));
message.insert("subsystem", XPCObject::from(3 as u64));
message.insert("routine", XPCObject::from(815 as u64));
message.insert("legacy", XPCObject::from(true));

let xpc_object: XPCObject = message.into();
```

Call `xpc_pipe_routine` and receive `Result<XPCObject, XPCError>`:

```rust
let xpc_object: XPCObject = message.into();

match xpc_object.pipe_routine() {
    Ok(xpc_object) => { /* do stuff and things */ },
    Err(XPCError::PipeError(err)) => { /* err is a string w/strerror(errno) */ }
}
```

The response is likely an XPC dictionary -- go back to a HashMap:

```rust
let xpc_object: XPCObject = message.into();
let response: Result<XPCDictionary, XPCError> = xpc_object
    .pipe_routine()
    .and_then(|r| r.try_into());

let XPCDictionary(hm) = response.unwrap();
let whatever = hm.get("...");
```

Response dictionaries can be nested, so `XPCDictionary` has a helper included for this scenario:

```rust
let xpc_object: XPCObject = message.into();

// A string: either "Aqua", "StandardIO", "Background", "LoginWindow", "System"
let response: Result<String, XPCError> = xpc_object
    .pipe_routine()
    .and_then(|r: XPCObject| r.try_into());
    .and_then(|d: XPCDictionary| d.get(&["service", "LimitLoadToSessionType"])
    .and_then(|lltst: XPCObject| lltst.xpc_value());
```

Or, retrieve the `service` key (a child XPC Dictionary) from this response:

```rust
let xpc_object: XPCObject = message.into();

// A string: either "Aqua", "StandardIO", "Background", "LoginWindow", "System"
let response: Result<XPCDictionary, XPCError> = xpc_object
    .pipe_routine()
    .and_then(|r: XPCObject| r.try_into());
    .and_then(|d: XPCDictionary| d.get_as_dictionary(&["service"]);

let XPCDictionary(hm) = response.unwrap();
let whatever = hm.get("...");
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

// Use as _xpc_type_shmem argument in XPCDictionary
let response = XPCDictionary::new()
    .extend(&DUMPSTATE)
    .entry("shmem", shmem.xpc_object.clone())
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
- After all, it is Apple's launchd :>)

Everything else (C) David Stancu & Contributors 2021
