use std::os::raw::c_int;

pub type csr_config_t = u32;

// https://github.com/apple/darwin-xnu/blob/main/bsd/kern/kern_csr.c
static CSR_ALLOW_UNTRUSTED_KEXTS: u32 = 1;

extern "C" {
    pub fn csr_check(mask: csr_config_t) -> c_int;
}

pub fn sip_enabled() -> bool {
    let res = unsafe { csr_check(CSR_ALLOW_UNTRUSTED_KEXTS) };
    res != 0
}
