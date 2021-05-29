# launchk

[![Rust](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/mach-kernel/launchk/actions/workflows/rust.yml)

A [Cursive](https://github.com/gyscos/cursive) TUI that makes XPC queries & helps manage launchd jobs.

Should work on macOS 10.10+ according to the availability sec. [in the docs](https://developer.apple.com/documentation/xpc?language=objc).

<img src="https://i.imgur.com/JYzEkx1.gif" width="600">

#### Features

- Poll XPC for jobs and display changes as they happen
- Filter by `LaunchAgents` and `LaunchDaemons` in scopes:
  - System (/System/Library/)
  - Global (/Library)
  - User (~/) 
- fsnotify detection for new plists added to above directories
- load
- unload
- dumpstate (opens in `$PAGER`)
- dumpjpcategory (opens in `$PAGER`)
- procinfo (opens in `$PAGER`, does not require root!)
- `:edit` -> Open plist in `$EDITOR`, defaulting to `vim`. Supports binary plists -> shown as XML for edit, then marshalled back into binary format on save.

#### xpc-sys

While building launchk all of the XPC convenience glue was placed in `xpc-sys`. [See its docs here](xpc-sys/README.md).

### Credits

A big thanks to these open source projects and general resources:

- [block](https://crates.io/crates/block) Obj-C block support, necessary for any XPC function taking `xpc_*_applier_t`  
- [Cursive](https://github.com/gyscos/cursive) TUI  
- [tokio](https://github.com/tokio-rs/tokio) ASIO  
- [plist](https://crates.io/crates/plist) Parsing & validation for XML and binary plists  
- [notify](https://docs.rs/notify/4.0.16/notify/) fsnotify  
- [bitflags](https://docs.rs/bitflags/1.2.1/bitflags/)  
- [Apple Developer XPC services](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingXPCServices.html)  
- [Apple Developer XPC API reference](https://developer.apple.com/documentation/xpc?language=objc)  
- [MOXIL / launjctl](http://newosxbook.com/articles/jlaunchctl.html)  
- [geosnow - A Long Evening With macOS' sandbox](https://geosn0w.github.io/A-Long-Evening-With-macOS%27s-Sandbox/)  
- [Bits of launchd - @5aelo](https://saelo.github.io/presentations/bits_of_launchd.pdf)  
- [Audit tokens explained (e.g. ASID)](https://knight.sc/reverse%20engineering/2020/03/20/audit-tokens-explained.html)  
- [objc.io XPC guide](https://www.objc.io/issues/14-mac/xpc/)  
- The various source links found in comments, from Chrome's sandbox and other headers with definitions for private API functions.
- Last but not least, this is Apple's launchd after all, right :>)? I did not know systemd was inspired by launchd until I read [this HN comment](https://news.ycombinator.com/item?id=2565780), which sent me down this eventual rabbit hole :)  

Everything else (C) David Stancu & Contributors 2021