use std::os::raw::c_int;
use std::collections::HashMap;

pub type csr_config_t = u32;

// https://github.com/apple/darwin-xnu/blob/main/bsd/kern/kern_csr.c
pub static CSR_ALLOW_UNTRUSTED_KEXTS: csr_config_t = (1 << 0);
pub static CSR_ALLOW_UNRESTRICTED_FS: csr_config_t = (1 << 1);
pub static CSR_ALLOW_TASK_FOR_PID: csr_config_t = (1 << 2);
pub static CSR_ALLOW_KERNEL_DEBUGGER: csr_config_t = (1 << 3);
pub static CSR_ALLOW_APPLE_INTERNAL: csr_config_t = (1 << 4);
pub static CSR_ALLOW_DESTRUCTIVE_DTRACE: csr_config_t = (1 << 5) /* name deprecated */;
pub static CSR_ALLOW_UNRESTRICTED_DTRACE: csr_config_t = (1 << 5);
pub static CSR_ALLOW_UNRESTRICTED_NVRAM: csr_config_t = (1 << 6);
pub static CSR_ALLOW_DEVICE_CONFIGURATION: csr_config_t = (1 << 7);
pub static CSR_ALLOW_ANY_RECOVERY_OS: csr_config_t = (1 << 8);
pub static CSR_ALLOW_UNAPPROVED_KEXTS: csr_config_t = (1 << 9);
pub static CSR_ALLOW_EXECUTABLE_POLICY_OVERRIDE: csr_config_t = (1 << 10);
pub static CSR_ALLOW_UNAUTHENTICATED_ROOT: csr_config_t = (1 << 11);

extern "C" {
    pub fn csr_check(mask: csr_config_t) -> c_int;
}

lazy_static! {
    pub static ref CSR_STATUS: HashMap<csr_config_t, bool> = unsafe {
        vec![
            // Can extrapolate SIP status from CSR_ALLOW_UNTRUSTED_KEXTS
            (CSR_ALLOW_UNTRUSTED_KEXTS, csr_check(CSR_ALLOW_UNTRUSTED_KEXTS) == 0),
            (CSR_ALLOW_UNRESTRICTED_FS, csr_check(CSR_ALLOW_UNRESTRICTED_FS) == 0),
            (CSR_ALLOW_TASK_FOR_PID, csr_check(CSR_ALLOW_TASK_FOR_PID) == 0),
            (CSR_ALLOW_KERNEL_DEBUGGER, csr_check(CSR_ALLOW_KERNEL_DEBUGGER) == 0),
            (CSR_ALLOW_APPLE_INTERNAL, csr_check(CSR_ALLOW_APPLE_INTERNAL) == 0),
            (CSR_ALLOW_DESTRUCTIVE_DTRACE, csr_check(CSR_ALLOW_DESTRUCTIVE_DTRACE) == 0),
            (CSR_ALLOW_UNRESTRICTED_DTRACE, csr_check(CSR_ALLOW_UNRESTRICTED_DTRACE) == 0),
            (CSR_ALLOW_UNRESTRICTED_NVRAM, csr_check(CSR_ALLOW_UNRESTRICTED_NVRAM) == 0),
            (CSR_ALLOW_DEVICE_CONFIGURATION, csr_check(CSR_ALLOW_DEVICE_CONFIGURATION) == 0),
            (CSR_ALLOW_ANY_RECOVERY_OS, csr_check(CSR_ALLOW_ANY_RECOVERY_OS) == 0),
            (CSR_ALLOW_UNAPPROVED_KEXTS, csr_check(CSR_ALLOW_UNAPPROVED_KEXTS) == 0),
            (CSR_ALLOW_EXECUTABLE_POLICY_OVERRIDE, csr_check(CSR_ALLOW_EXECUTABLE_POLICY_OVERRIDE) == 0),
            (CSR_ALLOW_UNAUTHENTICATED_ROOT, csr_check(CSR_ALLOW_UNAUTHENTICATED_ROOT) == 0),
        ].iter().cloned().collect()
    };
}
