use std::collections::HashMap;
use std::os::raw::c_int;

pub type csr_config_t = u32;

// https://github.com/apple/darwin-xnu/blob/main/bsd/kern/kern_csr.c
bitflags! {
    pub struct CsrConfig: csr_config_t {
        const ALLOW_UNTRUSTED_KEXTS = 1 << 0;
        const ALLOW_UNRESTRICTED_FS = 1 << 1;
        const ALLOW_TASK_FOR_PID = 1 << 2;
        const ALLOW_KERNEL_DEBUGGER = 1 << 3;
        const ALLOW_APPLE_INTERNAL = 1 << 4;
        const ALLOW_DESTRUCTIVE_DTRACE = 1 << 5 /* name deprecated */;
        const ALLOW_UNRESTRICTED_DTRACE = 1 << 5;
        const ALLOW_UNRESTRICTED_NVRAM = 1 << 6;
        const ALLOW_DEVICE_CONFIGURATION = 1 << 7;
        const ALLOW_ANY_RECOVERY_OS = 1 << 8;
        const ALLOW_UNAPPROVED_KEXTS = 1 << 9;
        const ALLOW_EXECUTABLE_POLICY_OVERRIDE = 1 << 10;
        const ALLOW_UNAUTHENTICATED_ROOT = 1 << 11;
    }
}

extern "C" {
    /// 0 if has mask
    pub fn csr_check(mask: csr_config_t) -> c_int;
}