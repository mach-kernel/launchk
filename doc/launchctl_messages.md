# launchctl messages

XPC messages sent by `launchctl` for some basic commands as presented by `xpc_copy_description`.

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

Response:

```
    	"EnableTransactions" => <bool: 0x7fff9464d490>: true
    	"Sockets" => <dictionary: 0x7fcc6462dc80> { count = 1, transaction: 0, voucher = 0x0, contents =
    		"Listeners" => <array: 0x7fcc64630370> { count = 1, capacity = 1, contents =
    			0: <fd: 0x7fcc6462e3c0> { type = (invalid descriptor), path = (invalid path) }
    		}
    	}
    	"LimitLoadToSessionType" => <string: 0x7fcc6462dd90> { length = 6, contents = "System" }
    	"Label" => <string: 0x7fcc6462de00> { length = 17, contents = "com.apple.usbmuxd" }
    	"OnDemand" => <bool: 0x7fff9464d4b0>: false
    	"LastExitStatus" => <int64: 0x97301c3994c391b1>: 0
    	"PID" => <int64: 0x97301c3994c9c1b1>: 165
    	"Program" => <string: 0x7fcc64631050> { length = 85, contents = "/System/Library/PrivateFrameworks/MobileDevice.framework/Versions/A/Resources/usbmuxd" }
    	"ProgramArguments" => <array: 0x7fcc6462dea0> { count = 2, capacity = 2, contents =
    		0: <string: 0x7fcc6462df30> { length = 85, contents = "/System/Library/PrivateFrameworks/MobileDevice.framework/Versions/A/Resources/usbmuxd" }
    		1: <string: 0x7fcc64630f80> { length = 8, contents = "-launchd" }
    	}
    }
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

#### `launchctl disable system/com.apple.FontWorker`

- Must run as root
- Type `1`

```
<dictionary: 0x100204480> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xe11a6f0157820409>: 3
	"handle" => <uint64: 0xe11a6f0157823409>: 0
	"routine" => <uint64: 0xe11a6f0157b0a409>: 809
	"name" => <string: 0x100204640> { length = 20, contents = "com.apple.FontWorker" }
	"type" => <uint64: 0xe11a6f0157822409>: 1
	"names" => <array: 0x1002046a0> { count = 1, capacity = 8, contents =
		0: <string: 0x100204760> { length = 20, contents = "com.apple.FontWorker" }
	}
```

#### `launchctl enable system/com.apple.FontWorker`

- This has a different `LimitLoadToSessionType` set as background, wanted to see if `type` would change

```
<dictionary: 0x100404140> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xf489c28b42fa66cd>: 3
	"handle" => <uint64: 0xf489c28b42fa56cd>: 0
	"routine" => <uint64: 0xf489c28b42c8d6cd>: 808
	"name" => <string: 0x100404310> { length = 20, contents = "com.apple.FontWorker" }
	"type" => <uint64: 0xf489c28b42fa46cd>: 1
	"names" => <array: 0x100404370> { count = 1, capacity = 8, contents =
		0: <string: 0x1004045b0> { length = 20, contents = "com.apple.FontWorker" }
	}
```

Using a `gui/` domain target:

```	
<dictionary: 0x100404080> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0x199d6ee9dd75252d>: 3
	"handle" => <uint64: 0x199d6ee9dd6a452d>: 501
	"routine" => <uint64: 0x199d6ee9dd47952d>: 808
	"name" => <string: 0x1004042c0> { length = 17, contents = "com.docker.vmnetd" }
	"type" => <uint64: 0x199d6ee9dd75952d>: 8
	"names" => <array: 0x100404320> { count = 1, capacity = 8, contents =
		0: <string: 0x1004043e0> { length = 17, contents = "com.docker.vmnetd" }
	}
```

1: System
2: User
8: Login (GUI)?

#### `launchctl dumpstate`

```
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x100504420> { count = 5, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0x9477db3970fa9c35>: 3
	"handle" => <uint64: 0x9477db3970faac35>: 0
	"shmem" => <shmem: 0x1005045e0>: 20971520 bytes (5121 pages)
	"routine" => <uint64: 0x9477db3970ce8c35>: 834
	"type" => <uint64: 0x9477db3970fabc35>: 1
(int) $0 = 328
```

Get the shmem key from the dictionary, map the shmem region, then continue so it can be filled, then it can be read from: 

```
expr void * $my_shmem = ((void *) xpc_dictionary_get_value($rsi, "shmem"));
expr void * $my_region = 0; 
expr size_t $my_shsize = (size_t) xpc_shmem_map($my_shmem, &$my_region);
c
(lldb) mem read $my_region $my_region+100
0x106580000: 63 6f 6d 2e 61 70 70 6c 65 2e 78 70 63 2e 6c 61  com.apple.xpc.la
0x106580010: 75 6e 63 68 64 2e 64 6f 6d 61 69 6e 2e 73 79 73  unchd.domain.sys
0x106580020: 74 65 6d 20 3d 20 7b 0a 09 74 79 70 65 20 3d 20  tem = {..type =
0x106580030: 73 79 73 74 65 6d 0a 09 68 61 6e 64 6c 65 20 3d  system..handle =
0x106580040: 20 30 0a 09 61 63 74 69 76 65 20 63 6f 75 6e 74   0..active count
0x106580050: 20 3d 20 35 37 35 0a 09 6f 6e 2d 64 65 6d 61 6e   = 575..on-deman
0x106580060: 64 20 63 6f                                      d co
```

#### `launchctl procinfo 7578`

This makes a whole _bunch_ of XPC calls! First it enumerates some ports:

```
2021-05-18 20:44:57.098359-0400 launchctl[7578:42112] [All] launchctl procinfo: launchctl procinfo 7475
program path = /usr/local/Cellar/redis/6.2.1/bin/redis-server
mach info = {
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x1006041b0> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a742d>: 3
	"handle" => <uint64: 0xc03dd40d963a442d>: 0
	"routine" => <uint64: 0xc03dd40d9609242d>: 822
	"process" => <int64: 0xc03dd40d97e3e43d>: 7578
	"name" => <uint64: 0xc03dd40d96ea342d>: 3335
	"type" => <uint64: 0xc03dd40d963a542d>: 1
(int) $6 = 360
```

```
}	task-kernel port = 0xd07 (unknown)
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x100305420> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a742d>: 3
	"handle" => <uint64: 0xc03dd40d963a442d>: 0
	"routine" => <uint64: 0xc03dd40d9609242d>: 822
	"process" => <int64: 0xc03dd40d97e3e43d>: 7578
	"name" => <uint64: 0xc03dd40d971a742d>: 4611
	"type" => <uint64: 0xc03dd40d963a542d>: 1
(int) $7 = 360
```

```
}	task-host port = 0x1203 (unknown)
Process 7578 stopped
* thread #1, queue = 'com.apple.main-thread', stop reason = breakpoint 1.3
    frame #0: 0x00007fff2005e841 libxpc.dylib`xpc_pipe_routine_with_flags
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x100204410> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a742d>: 3
	"handle" => <uint64: 0xc03dd40d963a442d>: 0
	"routine" => <uint64: 0xc03dd40d9609242d>: 822
	"process" => <int64: 0xc03dd40d97e3e43d>: 7578
	"name" => <uint64: 0xc03dd40d975a742d>: 5635
	"type" => <uint64: 0xc03dd40d963a542d>: 1
```

```
}	task-name port = 0x1603 (unknown)
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x1004049b0> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a742d>: 3
	"handle" => <uint64: 0xc03dd40d963a442d>: 0
	"routine" => <uint64: 0xc03dd40d9609242d>: 822
	"process" => <int64: 0xc03dd40d97e3e43d>: 7578
	"name" => <uint64: 0xc03dd40d977a742d>: 5123
	"type" => <uint64: 0xc03dd40d963a542d>: 1
(int) $9 = 360
```

```
}	task-bootstrap port = 0x1403 (unknown)
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x100305420> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a742d>: 3
	"handle" => <uint64: 0xc03dd40d963a442d>: 0
	"routine" => <uint64: 0xc03dd40d9609242d>: 822
	"process" => <int64: 0xc03dd40d97e3e43d>: 7578
	"name" => <uint64: 0xc03dd40d975a342d>: 5639
	"type" => <uint64: 0xc03dd40d963a542d>: 1
```

```
}	task-(null) port = 0x1607 (unknown)
(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x1004040c0> { count = 6, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a742d>: 3
	"handle" => <uint64: 0xc03dd40d963a442d>: 0
	"routine" => <uint64: 0xc03dd40d9609242d>: 822
	"process" => <int64: 0xc03dd40d97e3e43d>: 7578
	"name" => <uint64: 0xc03dd40d975af42d>: 5643
	"type" => <uint64: 0xc03dd40d963a542d>: 1
```

Now for our old shmem / stdout friend:

```
argument count = 3
argument vector = {
	[0] = /usr/local/opt/redis/bin/redis-server 127.0.0.1:6379
	[1] = XPC_FLAGS=1
	[2] = LOGNAME=mach
}
environment vector = {
	USER => mach
	HOME => /Users/mach
	SHELL => /bin/zsh
	TMPDIR => /var/folders/sl/4tlmgdgj60j2wgykq7q10pdw0000gn/T/
}
bsd proc info = {
	pid = 7475
	unique pid = 7475
	ppid = 1
	pgid = 7475
	status = stopped
	flags = 64-bit
	uid = 501
	svuid = 501
	ruid = 501
	gid = 20
	svgid = 20
	rgid = 20
	comm name = redis-server
	long name = redis-server
	controlling tty devnode = 0xffffffff
	controlling tty pgid = 0
}
audit info
	session id = 100006
	uid = 501
	success mask = 0x3000
	failure mask = 0x3000
	flags = has_graphic_access,has_tty,has_console_access,has_authenticated
sandboxed = no
container = (no container)

responsible pid = 7475
responsible unique pid = 7475
responsible path = /usr/local/Cellar/redis/6.2.1/bin/redis-server

pressured exit info = {
	dirty state tracked = 0
	dirty = 0
	pressured-exit capable = 0
}

jetsam priority = 3: background
jetsam memory limit = -1
jetsam state = (normal memory state)

entitlements = (no entitlements)

code signing info = (none)

(lldb) p printf("%s",(char*)  xpc_copy_description($rsi))
<dictionary: 0x1006041b0> { count = 4, transaction: 0, voucher = 0x0, contents =
	"subsystem" => <uint64: 0xc03dd40d963a642d>: 2
	"fd" => <fd: 0x100604850> { type = (invalid descriptor), path = /dev/ttys003 }
	"routine" => <uint64: 0xc03dd40d9616042d>: 708
	"pid" => <int64: 0xc03dd40d97e9743d>: 7475
(int) $14 = 302
```
