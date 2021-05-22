use crate::{vm_address_t, mach_port_t, vm_size_t, vm_allocate, rs_strerror, vm_deallocate, mach_task_self_};
use std::sync::Arc;
use crate::objects::xpc_object::XPCObject;
use std::os::raw::c_int;
use crate::objects::xpc_error::XPCError;
use std::ptr::null_mut;

pub struct XPCShmem {
    pub task: mach_port_t,
    pub size: vm_size_t,
    pub region: Arc<*mut vm_address_t>,
    pub xpc_object: XPCObject,
}

impl XPCShmem {
    pub fn new(
        task: mach_port_t,
        size: vm_size_t,
        flags: c_int,
    ) -> Result<XPCShmem, XPCError> {
        let mut region: *mut vm_address_t = null_mut();
        let err = unsafe {
            vm_allocate(
                task,
                region,
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
                region: Arc::new(region),
                xpc_object: Default::default()
            })
        }
    }

    pub fn new_task_self(
        size: vm_size_t,
        flags: c_int,
    ) -> Result<XPCShmem, XPCError> {
        unsafe { Self::new(mach_task_self_, size, flags) }
    }
}

impl Drop for XPCShmem {
    fn drop(&mut self) {
        let XPCShmem { size, task, region, .. } = self;
        if **region == null_mut() {
            return;
        }

        let refs = Arc::strong_count(&region);
        if refs > 1 {
            log::warn!("vm_allocated region {:p} still has {} refs, cannot vm_deallocate", *region, refs);
            return;
        }

        unsafe {
            vm_deallocate(*task, ***region, *size)
        };
    }
}