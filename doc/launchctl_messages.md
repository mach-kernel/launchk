# launchctl messages

This is a handy reference of XPC messages sent by `launchctl` for some basic commands, as presented by `xpc_copy_description`.

#### `launchctl print system`

```
<dictionary: 0x100704180> { count = 5, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x823dd3881d7aa785>: 3
        "fd" => <fd: 0x100404120> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x823dd3881d7a9785>: 0
        "routine" => <uint64: 0x823dd3881d495785>: 828
        "type" => <uint64: 0x823dd3881d7a8785>: 1
```

#### `launchctl print system/com.apple.lskdd`

```
<dictionary: 0x1004045c0> { count = 6, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x436173f5352414b1>: 2
        "fd" => <fd: 0x1004041b0> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x436173f5352434b1>: 0
        "routine" => <uint64: 0x436173f5350874b1>: 708
        "name" => <string: 0x100404390> { length = 15, contents = "com.apple.lskdd" }
        "type" => <uint64: 0x436173f5352424b1>: 1
```

#### `launchctl print user/501`

```
<dictionary: 0x1007040c0> { count = 5, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0xab418bfb6970e5f5>: 3
        "fd" => <fd: 0x100704290> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0xab418bfb696f85f5>: 501
        "routine" => <uint64: 0xab418bfb694315f5>: 828
        "type" => <uint64: 0xab418bfb6970f5f5>: 2
```

#### `launchctl print user/501/com.apple.podcasts.PodcastContentService`

```
<dictionary: 0x100604080> { count = 6, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x7d3ca0d9be8bf1e1>: 2
        "fd" => <fd: 0x100704120> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x7d3ca0d9be9481e1>: 501
        "routine" => <uint64: 0x7d3ca0d9bea791e1>: 708
        "name" => <string: 0x1007041e0> { length = 40, contents = "com.apple.podcasts.PodcastContentService" }
        "type" => <uint64: 0x7d3ca0d9be8bf1e1>: 2
```

#### `launchctl print login/100006`

```
<dictionary: 0x1004047b0> { count = 5, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0xf11a5eddfd695dfd>: 3
        "fd" => <fd: 0x1004042b0> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0xf11a5edde5030dfd>: 100006
        "routine" => <uint64: 0xf11a5eddfd5aadfd>: 828
        "type" => <uint64: 0xf11a5eddfd695dfd>: 3
```

#### `launchctl print login/100006/com.apple.assistantd`

```
<dictionary: 0x100504280> { count = 6, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x273c5d22dba1254d>: 2
        "fd" => <fd: 0x100504490> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x273c5d22c3cb654d>: 100006
        "routine" => <uint64: 0x273c5d22db8d454d>: 708
        "name" => <string: 0x100504210> { length = 20, contents = "com.apple.assistantd" }
        "type" => <uint64: 0x273c5d22dba1354d>: 3
```

#### `launchctl print gui/501/com.apple.usernoted`

```
<dictionary: 0x1003042b0> { count = 6, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x2a0ec3b9a05b283>: 2
        "fd" => <fd: 0x100304460> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x2a0ec3b9a1ac283>: 501
        "routine" => <uint64: 0x2a0ec3b9a29d283>: 708
        "name" => <string: 0x100304400> { length = 19, contents = "com.apple.usernoted" }
        "type" => <uint64: 0x2a0ec3b9a051283>: 8
```

#### `launchctl print gui/501`

```
<dictionary: 0x1003042b0> { count = 5, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x85f95cc358039f0b>: 3
        "fd" => <fd: 0x100304170> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x85f95cc3581cff0b>: 501
        "routine" => <uint64: 0x85f95cc358306f0b>: 828
        "type" => <uint64: 0x85f95cc358032f0b>: 8
```

#### `launchctl print pid/1613/com.apple.security.pboxd`

```
<dictionary: 0x1004045b0> { count = 6, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0x68228cc69498dcd5>: 2
        "fd" => <fd: 0x100404650> { type = (invalid descriptor), path = /dev/ttys007 }
        "handle" => <uint64: 0x68228cc694fc2cd5>: 1613
        "routine" => <uint64: 0x68228cc694b4bcd5>: 708
        "name" => <string: 0x100404500> { length = 24, contents = "com.apple.security.pboxd" }
        "type" => <uint64: 0x68228cc69498acd5>: 5
```

#### `launchctl dumpjpcategory`

```
}<dictionary: 0x1020041a0> { count = 5, transaction: 0, voucher = 0x0, contents =
        "subsystem" => <uint64: 0xcfd74598a170ebaf>: 3
        "fd" => <fd: 0x100205b70> { type = (invalid descriptor), path = /dev/ttys009 }
        "handle" => <uint64: 0xcfd74598a170dbaf>: 0
        "routine" => <uint64: 0xcfd74598a1448baf>: 837
        "type" => <uint64: 0xcfd74598a170cbaf>: 1
```

#### `launchctl print-cache`

- `handle` is the PID

```
}<dictionary: 0x1007042f0> { count = 4, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xdaa38f2c75b6474f>: 3
	"handle" => <uint64: 0xdaa38f2c7531c74f>: 2171
	"routine" => <uint64: 0xdaa38f2c7584474f>: 803
	"type" => <uint64: 0xdaa38f2c75b6274f>: 5
```

#### `launchctl list`

The two messages below are produced by the command. Not sure what `type` is here.

As the current user:

```
<dictionary: 0x1002054f0> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xf7cad99ebbc6520f>: 3
	"handle" => <uint64: 0xf7cad99ebbc6620f>: 0
	"routine" => <uint64: 0xf7cad99ebbf4920f>: 815
	"type" => <uint64: 0xf7cad99ebbc6120f>: 7
	"legacy" => <bool: 0x7fff800130b0>: true
	"domain-port" => <mach send right: 0x100205e50> { name = 1799, right = send, urefs = 5 }
```

As root:

```
<dictionary: 0x1004042b0> { count = 5, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xd9db7e06388e848b>: 3
	"handle" => <uint64: 0xd9db7e06388eb48b>: 0
	"routine" => <uint64: 0xd9db7e0638bc448b>: 815
	"type" => <uint64: 0xd9db7e06388ea48b>: 1
	"legacy" => <bool: 0x7fff800130b0>: true
```

Maybe it works like `print`? With types 1-8 and `strerror` for XPC responses with an `error` key:

```
type 1
ok
type 2
strerr Operation not permitted (this works if you are root)
type 3
strerr Domain does not support specified action
type 4
strerr Domain does not support specified action
type 5
strerr Could not find specified domain
type 6
ok
type 7
ok
type 8
strerr Domain does not support specified action
```

#### `launchctl unload ~/Library/LaunchAgents/homebrew.mxcl.elasticsearch.plist`

```
<dictionary: 0x100305170> { count = 11, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xf048e68836ce456d>: 3
	"handle" => <uint64: 0xf048e68836ce756d>: 0
	"legacy-load" => <bool: 0x7fff800130b0>: true
	"routine" => <uint64: 0xf048e68836fc656d>: 801
	"paths" => <array: 0x100305740> { count = 1, capacity = 8, contents =
		0: <string: 0x100305850> { length = 66, contents = "/Users/mach/Library/LaunchAgents/homebrew.mxcl.elasticsearch.plist" }
	}
	"no-einprogress" => <bool: 0x7fff800130b0>: true
	"disable" => <bool: 0x7fff800130d0>: false
	"type" => <uint64: 0xf048e68836ce056d>: 7
	"legacy" => <bool: 0x7fff800130b0>: true
	"session" => <string: 0x100305630> { length = 4, contents = "Aqua" }
	"domain-port" => <mach send right: 0x100304fa0> { name = 1799, right = send, urefs = 5 }
```

Type seems same even if trying from `/Library`:

```
<dictionary: 0x100604420> { count = 11, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0x45e43765185d939b>: 3
	"handle" => <uint64: 0x45e43765185da39b>: 0
	"legacy-load" => <bool: 0x7fff800130b0>: true
	"routine" => <uint64: 0x45e43765186fb39b>: 801
	"paths" => <array: 0x100604680> { count = 1, capacity = 8, contents =
		0: <string: 0x100604770> { length = 57, contents = "/Library/LaunchDaemons/com.adobe.ARMDC.Communicator.plist" }
	}
	"no-einprogress" => <bool: 0x7fff800130b0>: true
	"disable" => <bool: 0x7fff800130d0>: false
	"type" => <uint64: 0x45e43765185dd39b>: 7
	"legacy" => <bool: 0x7fff800130b0>: true
	"session" => <string: 0x1006045f0> { length = 4, contents = "Aqua" }
	"domain-port" => <mach send right: 0x1006044b0> { name = 1799, right = send, urefs = 5 }
```

As root:

```
<dictionary: 0x100304550> { count = 9, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xb2614a0bcbc04b91>: 3
	"handle" => <uint64: 0xb2614a0bcbc07b91>: 0
	"legacy-load" => <bool: 0x7fff800130b0>: true
	"routine" => <uint64: 0xb2614a0bcbf26b91>: 801
	"paths" => <array: 0x100304730> { count = 1, capacity = 8, contents =
		0: <string: 0x100304820> { length = 57, contents = "/Library/LaunchDaemons/com.adobe.ARMDC.Communicator.plist" }
	}
	"no-einprogress" => <bool: 0x7fff800130b0>: true
	"disable" => <bool: 0x7fff800130d0>: false
	"type" => <uint64: 0xb2614a0bcbc06b91>: 1
	"legacy" => <bool: 0x7fff800130b0>: true
```

#### `launchctl load ~/Library/LaunchAgents/homebrew.mxcl.elasticsearch.plist`

```
<dictionary: 0x1006044b0> { count = 10, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xe9f0c2b9827fda95>: 3
	"handle" => <uint64: 0xe9f0c2b9827fea95>: 0
	"legacy-load" => <bool: 0x7fff800130b0>: true
	"enable" => <bool: 0x7fff800130d0>: false
	"routine" => <uint64: 0xe9f0c2b9824dea95>: 800
	"paths" => <array: 0x1006046a0> { count = 1, capacity = 8, contents =
		0: <string: 0x1006047b0> { length = 66, contents = "/Users/mach/Library/LaunchAgents/homebrew.mxcl.elasticsearch.plist" }
	}
	"type" => <uint64: 0xe9f0c2b9827f9a95>: 7
	"legacy" => <bool: 0x7fff800130b0>: true
	"session" => <string: 0x100604610> { length = 4, contents = "Aqua" }
	"domain-port" => <mach send right: 0x100604540> { name = 1799, right = send, urefs = 5 }
```

#### `launchctl disable user/501/homebrew.mxcl.postgresql`

- postgresql runs in the `Aqua` domain
- Interesting that both `name` and `names` are sent!

```
<dictionary: 0x100404340> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0x2d24e480c52e0073>: 3
	"handle" => <uint64: 0x2d24e480c5316073>: 501
	"routine" => <uint64: 0x2d24e480c51ca073>: 809
	"name" => <string: 0x100404440> { length = 24, contents = "homebrew.mxcl.postgresql" }
	"type" => <uint64: 0x2d24e480c52e1073>: 2
	"names" => <array: 0x1004044a0> { count = 1, capacity = 8, contents =
		0: <string: 0x100404560> { length = 24, contents = "homebrew.mxcl.postgresql" }
	}
```


#### `launchctl enable user/501/homebrew.mxcl.postgresql`

```
<dictionary: 0x1004042b0> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xd49d6ee83bb4aaf3>: 3
	"handle" => <uint64: 0xd49d6ee83babcaf3>: 501
	"routine" => <uint64: 0xd49d6ee83b861af3>: 808
	"name" => <string: 0x1004043f0> { length = 24, contents = "homebrew.mxcl.postgresql" }
	"type" => <uint64: 0xd49d6ee83bb4baf3>: 2
	"names" => <array: 0x100404450> { count = 1, capacity = 8, contents =
		0: <string: 0x100404520> { length = 24, contents = "homebrew.mxcl.postgresql" }
	}
```
