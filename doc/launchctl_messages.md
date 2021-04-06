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
