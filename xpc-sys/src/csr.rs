use std::{os::raw::c_int, fmt, fmt::Formatter};

pub type csr_config_t = u32;

// https://github.com/apple/darwin-xnu/blob/main/bsd/kern/kern_csr.c
bitflags! {
    #[derive(PartialEq, Eq)]
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

impl fmt::Debug for CsrConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            CsrConfig::ALLOW_UNTRUSTED_KEXTS => write!(f, "ALLOW_UNTRUSTED_KEXTS"),
            CsrConfig::ALLOW_UNRESTRICTED_FS => write!(f, "ALLOW_UNRESTRICTED_FS"),
            CsrConfig::ALLOW_TASK_FOR_PID => write!(f, "ALLOW_TASK_FOR_PID"),
            CsrConfig::ALLOW_KERNEL_DEBUGGER => write!(f, "ALLOW_KERNEL_DEBUGGER"),
            CsrConfig::ALLOW_APPLE_INTERNAL => write!(f, "ALLOW_APPLE_INTERNAL"),
            CsrConfig::ALLOW_DESTRUCTIVE_DTRACE => write!(f, "ALLOW_DESTRUCTIVE_DTRACE | ALLOW_UNRESTRICTED_DTRACE"),
            CsrConfig::ALLOW_UNRESTRICTED_NVRAM => write!(f, "ALLOW_UNRESTRICTED_NVRAM"),
            CsrConfig::ALLOW_DEVICE_CONFIGURATION => write!(f, "ALLOW_DEVICE_CONFIGURATION"),
            CsrConfig::ALLOW_ANY_RECOVERY_OS => write!(f, "ALLOW_ANY_RECOVERY_OS"),
            CsrConfig::ALLOW_UNAPPROVED_KEXTS => write!(f, "ALLOW_UNAPPROVED_KEXTS"),
            CsrConfig::ALLOW_EXECUTABLE_POLICY_OVERRIDE => write!(f, "ALLOW_EXECUTABLE_POLICY_OVERRIDE"),
            CsrConfig::ALLOW_UNAUTHENTICATED_ROOT => write!(f, "ALLOW_UNAUTHENTICATED_ROOT"),
            _ => Ok(())
        }
    }
}

extern "C" {
    /// 0 if has mask
    pub fn csr_check(mask: csr_config_t) -> c_int;
}
