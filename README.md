# launchk

[![Rust](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml)

A WIP [Cursive](https://github.com/gyscos/cursive) TUI that makes XPC queries & helps manage launchd jobs.

Should work on macOS 10.10+ according to the availability sec. [in the docs](https://developer.apple.com/documentation/xpc?language=objc).

<img src="https://i.imgur.com/JYzEkx1.gif" width="600">

#### Features

- Poll XPC for jobs and display changes as they happen
- Filter by system (/System/Library/), "global" (/Library), user (~/) `LaunchAgents` and `LaunchDaemons`
- fsnotify detection for new plists added to above directories
- `:load/:unload` -> `launchctl load/unload`
- `:edit` -> Open plist in `$EDITOR`, defaulting to `vim` (TODO binary plist support)

### xpc-sys crate

There is some "convenience glue" for dealing with XPC objects. Eventually, this will be broken out into its own crate. Most of the tests (for now) are written around not breaking data going across the FFI barrier.

XPCObject wraps `xpc_object_t` in an `Arc`. `Drop` will invoke `xpc_release()` on objects being dropped with no other [strong refs](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.strong_count).

#### xpc_pipe_routine

Form your messages as a HashMap:

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

#### Making XPC Objects

Make XPC objects for anything with `From<T>`. From earlier example, even Mach ports:
```rust
let mut message: HashMap<&str, XPCObject> = HashMap::new();

message.insert(
    "domain-port",
    XPCObject::from(get_bootstrap_port() as mach_port_t),
);
```

Go from an XPC object to value via `TryXPCValue`. It checks your object's type via `xpc_get_type()` and yields a clear error if you're using the wrong type:
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
