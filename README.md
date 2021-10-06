# launchk

[![Rust](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml)

A [Cursive](https://github.com/gyscos/cursive) TUI that makes XPC queries & helps manage launchd jobs.

Should work on macOS 10.10+ according to the availability sec. [in the docs](https://developer.apple.com/documentation/xpc?language=objc).

<img src="https://user-images.githubusercontent.com/396039/120085936-6700a180-c0aa-11eb-8606-31dc6a3cbe99.gif" width="600">

#### Install

Binaries are available via brew tap:

```
brew tap mach-kernel/pkgs
brew install mach-kernel/pkgs/launchk
```

#### Features

Use `:` to enter command mode, `/` to enter filtering mode, and any of `sguadl` for filtering by [system, global, user, agent, daemon, loaded]. `Ctrl-U` to clear, `Tab` to complete, `Enter` to submit. 

- Poll XPC for jobs and display changes as they happen
- Filter by `LaunchAgents` and `LaunchDaemons` in scopes (fsnotify watched):
  - System (/System/Library/)
  - Global (/Library)
  - User (~/) 
- `load`
- `unload`
- `dumpstate` (opens in `$PAGER`)
- `dumpjpcategory` (opens in `$PAGER`)
- `procinfo` (opens in `$PAGER`, does not require root!)
- `edit` plist in `$EDITOR` with support for binary plists
- `csrinfo` show all CSR flags and their values

#### xpc-sys

While building launchk, XPC convenience glue was placed in `xpc-sys`. 

[[See its README here]](xpc-sys/README.md)

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
- [This HN comment](https://news.ycombinator.com/item?id=2565780) re history
- The various source links found in comments, from Chrome's sandbox and other headers with definitions for private API functions.
- After all, it is Apple's launchd :>)

Everything else (C) David Stancu & Contributors 2021
