use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_object::XPCObject;
use crate::{
    mach_port_t, mach_task_self_, rs_strerror, vm_address_t, vm_allocate, vm_deallocate, vm_size_t,
    xpc_shmem_create,
};
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::{ffi::c_void, sync::Arc};

#[derive(Debug, Clone)]
pub struct XPCShmem {
    pub task: mach_port_t,
    pub size: vm_size_t,
    pub region: *mut c_void,
    pub xpc_object: Arc<XPCObject>,
}

unsafe impl Send for XPCShmem {}

impl XPCShmem {
    pub fn new(task: mach_port_t, size: vm_size_t, flags: c_int) -> Result<XPCShmem, XPCError> {
        let mut region: *mut c_void = null_mut();
        let err = unsafe {
            vm_allocate(
                task,
                &mut region as *const _ as *mut vm_address_t,
                size,
                flags,
            )
        };

        if err > 0 {
            Err(XPCError::ShmemError(rs_strerror(err)))
        } else {
            Ok(XPCShmem {
                task,
                size,
                region,
                xpc_object: Arc::new(unsafe {
                    xpc_shmem_create(region as *mut c_void, size as u64).into()
                }),
            })
        }
    }

    pub fn new_task_self(size: vm_size_t, flags: c_int) -> Result<XPCShmem, XPCError> {
        unsafe { Self::new(mach_task_self_, size, flags) }
    }
}

impl Drop for XPCShmem {
    fn drop(&mut self) {
        let XPCShmem {
            size,
            task,
            region,
            xpc_object,
        } = self;
        if *region == null_mut() {
            return;
        }

        let refs = Arc::strong_count(xpc_object);
        if refs > 1 {
            log::warn!(
                "vm_allocated region {:p} still has {} refs, cannot vm_deallocate",
                *region,
                refs
            );
            return;
        }

        unsafe { vm_deallocate(*task, *region as vm_address_t, *size) };
    }
}
