# launchk

[![Rust](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml)

A small and WIP ncurses/cursive TUI around launchctl for managing Apple launchd.

Should work on macOS 10.10+ according to the availability sec. [in the docs](https://developer.apple.com/documentation/xpc?language=objc).

![](https://i.imgur.com/ly791YJ.gif)

### xpc-sys crate

There is some "convenience glue" for dealing with XPC objects. Eventually, this will be broken out into its own crate. Most of the tests (for now) are written around not breaking data going across the FFI barrier.

#### Dictionary

Form your messages as a HashMap:

```rust
let mut message: HashMap<&str, XPCObject> = HashMap::new();
message.insert("type", XPCObject::from(1 as u64));
message.insert("handle", XPCObject::from(0 as u64));
message.insert("subsystem", XPCObject::from(3 as u64));
message.insert("routine", XPCObject::from(815 as u64));
message.insert("legacy", XPCObject::from(true));

let my_ptr: xpc_object_t = XPCObject::from(message).as_ptr();
```

Or go from `xpc_type_dictionary` back to HashMap:

```rust
let mut reply: xpc_object_t = null_mut();
// xpc_pipe_routine(pipe, msg, &reply)

if let Ok(XPCDictionary(hm)) = reply.try_into() {
  // ...
}
```

#### Other types

Make XPC objects for anything with `From<T>`. From earlier example, even Mach ports:
```rust
let mut message: HashMap<&str, XPCObject> = HashMap::new();

message.insert(
    "domain-port",
    XPCObject::from(get_bootstrap_port() as mach_port_t),
);
```

Go from an XPC object to value via `TryXPCValue`. It checks your object's type via `xpc_get_type()` and yields a clear error if you're using the wrong type annotation:
```rust
#[test]
fn deserialize_as_wrong_type() {
    let an_i64: XPCObject = XPCObject::from(42 as i64);
    let as_u64: Result<u64, XPCValueError> = an_i64.xpc_value();
    assert_eq!(
        as_u64.err().unwrap(),
        XPCValueError("Cannot get int64 as uint64".to_string())
    );
}
```